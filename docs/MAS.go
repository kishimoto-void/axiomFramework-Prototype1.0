package main

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"math"
	"os"
	"reflect"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

// ============================================================
// 0. 決定論的 Canonical Serializer (完全キーソート保証)
// ============================================================

type LogicalClock int64

const MaxPayloadSize = 10 * 1024 * 1024 // 10MB 上限

func clampValue(v, min, max float64) float64 {
	if math.IsNaN(v) || math.IsInf(v, 0) {
		return min
	}
	return math.Max(min, math.Min(max, v))
}

func shortHash(s string, length int) string {
	if len(s) <= length {
		return s
	}
	return s[:length]
}

func deterministicErrorHash(err error) string {
	msg := "UNKNOWN_ERROR"
	if err != nil {
		msg = err.Error()
	}
	hash := sha256.Sum256([]byte("HASH_FALLBACK:" + msg))
	return "ERR_" + hex.EncodeToString(hash[:8])
}

// CanonicalMarshal は JSON の Map キーを完全決定論的に昇順ソートしてバイト化する
func CanonicalMarshal(v any) ([]byte, error) {
	normalized, err := normalizeForCanonical(v)
	if err != nil {
		return nil, err
	}
	return json.Marshal(normalized)
}

func normalizeForCanonical(v any) (any, error) {
	if v == nil {
		return nil, nil
	}
	val := reflect.ValueOf(v)
	switch val.Kind() {
	case reflect.Map:
		// Map のキーを文字列化して昇順ソート
		keys := val.MapKeys()
		keyStrs := make([]string, 0, len(keys))
		keyMap := make(map[string]reflect.Value, len(keys))
		for _, k := range keys {
			ks := fmt.Sprintf("%v", k.Interface())
			keyStrs = append(keyStrs, ks)
			keyMap[ks] = val.MapIndex(k)
		}
		sort.Strings(keyStrs)

		sortedMap := make(map[string]any, len(keyStrs))
		for _, ks := range keyStrs {
			normVal, err := normalizeForCanonical(keyMap[ks].Interface())
			if err != nil {
				return nil, err
			}
			sortedMap[ks] = normVal
		}
		return sortedMap, nil

	case reflect.Slice, reflect.Array:
		elemList := make([]any, val.Len())
		for i := 0; i < val.Len(); i++ {
			normVal, err := normalizeForCanonical(val.Index(i).Interface())
			if err != nil {
				return nil, err
			}
			elemList[i] = normVal
		}
		return elemList, nil

	case reflect.Struct:
		// Struct は一度 JSON 経由で Map に変換して再帰ソート
		raw, err := json.Marshal(v)
		if err != nil {
			return nil, err
		}
		var m map[string]any
		if err := json.Unmarshal(raw, &m); err != nil {
			return nil, err
		}
		return normalizeForCanonical(m)

	default:
		return v, nil
	}
}

// ============================================================
// 1. リソース制御 & プロトコル定義
// ============================================================

type ResourceBasis struct {
	MathematicianBasis int64
	PhysicistBasis     int64
	ObserverBasis      int64
}

func (r ResourceBasis) Validate() error {
	sum := r.MathematicianBasis + r.PhysicistBasis + r.ObserverBasis
	if sum != 1000 {
		return fmt.Errorf("ResourceBasis の合計は 1000 である必要があります (現在: %d)", sum)
	}
	return nil
}

func (r ResourceBasis) MathematicianRatio() float64 { return float64(r.MathematicianBasis) / 1000.0 }
func (r ResourceBasis) PhysicistRatio() float64     { return float64(r.PhysicistBasis) / 1000.0 }
func (r ResourceBasis) ObserverRatio() float64      { return float64(r.ObserverBasis) / 1000.0 }

var DefaultResourceBasis = ResourceBasis{
	MathematicianBasis: 300,
	PhysicistBasis:     300,
	ObserverBasis:      400,
}

type MasErrorCode string

const (
	ErrCodeBudgetExceeded        MasErrorCode = "BUDGET_EXCEEDED"
	ErrCodeWireSevered           MasErrorCode = "WIRESTRING_SEVERED"
	ErrCodeStateTransitionFailed MasErrorCode = "STATE_TRANSITION_FAILED"
)

type MasError struct {
	Code    MasErrorCode
	Message string
	Cause   error
}

func (e *MasError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf("[%s] %s: %v", e.Code, e.Message, e.Cause)
	}
	return fmt.Sprintf("[%s] %s", e.Code, e.Message)
}

func NewMasError(code MasErrorCode, msg string, cause error) *MasError {
	return &MasError{Code: code, Message: msg, Cause: cause}
}

var (
	WireMagicV4141  = []byte{'A', 'X', 'W', 'S'}
	WireVersion4141 = []byte{0x04, 0x01, 0x04, 0x01}
)

type WireFrame4141 struct {
	Sequence    uint64
	Clock       LogicalClock
	PayloadLen  uint32
	ChecksumSHA [32]byte
	Payload     []byte
}

func EncodeWireFrame4141(seq uint64, clock LogicalClock, payload []byte) ([]byte, error) {
	headerBuf := new(bytes.Buffer)
	_ = binary.Write(headerBuf, binary.BigEndian, WireMagicV4141)
	_ = binary.Write(headerBuf, binary.BigEndian, WireVersion4141)
	_ = binary.Write(headerBuf, binary.BigEndian, seq)
	_ = binary.Write(headerBuf, binary.BigEndian, int64(clock))
	_ = binary.Write(headerBuf, binary.BigEndian, uint32(len(payload)))
	headerBytes := headerBuf.Bytes()

	hasher := sha256.New()
	hasher.Write(headerBytes)
	hasher.Write(payload)
	checksum := hasher.Sum(nil)

	finalBuf := new(bytes.Buffer)
	finalBuf.Write(headerBytes)
	finalBuf.Write(checksum)
	finalBuf.Write(payload)
	return finalBuf.Bytes(), nil
}

func DecodeAndVerifyFrame4141(data []byte) (*WireFrame4141, error) {
	if len(data) < 60 {
		return nil, NewMasError(ErrCodeWireSevered, "ヘッダー長不足", nil)
	}
	buf := bytes.NewReader(data)
	headerRaw := make([]byte, 28)
	if _, err := io.ReadFull(buf, headerRaw); err != nil {
		return nil, NewMasError(ErrCodeWireSevered, "ヘッダー読み込み失敗", err)
	}

	frame := &WireFrame4141{}
	rHeader := bytes.NewReader(headerRaw)
	var magic, ver [4]byte
	var clockRaw int64
	_ = binary.Read(rHeader, binary.BigEndian, &magic)
	_ = binary.Read(rHeader, binary.BigEndian, &ver)
	_ = binary.Read(rHeader, binary.BigEndian, &frame.Sequence)
	_ = binary.Read(rHeader, binary.BigEndian, &clockRaw)
	_ = binary.Read(rHeader, binary.BigEndian, &frame.PayloadLen)
	frame.Clock = LogicalClock(clockRaw)

	if !bytes.Equal(magic[:], WireMagicV4141) || !bytes.Equal(ver[:], WireVersion4141) {
		return nil, NewMasError(ErrCodeWireSevered, "不整合ヘッダー", nil)
	}
	if frame.PayloadLen > MaxPayloadSize {
		return nil, NewMasError(ErrCodeWireSevered, fmt.Sprintf("ペイロードサイズ制限超過 (最大: %d, 要求: %d)", MaxPayloadSize, frame.PayloadLen), nil)
	}
	if len(data) != 60+int(frame.PayloadLen) {
		return nil, NewMasError(ErrCodeWireSevered, "フレーム全体長不一致", nil)
	}

	if _, err := io.ReadFull(buf, frame.ChecksumSHA[:]); err != nil {
		return nil, NewMasError(ErrCodeWireSevered, "チェックサム読み込み失敗", err)
	}
	payload := make([]byte, frame.PayloadLen)
	if _, err := io.ReadFull(buf, payload); err != nil {
		return nil, NewMasError(ErrCodeWireSevered, "ペイロード読み込み失敗", err)
	}

	hasher := sha256.New()
	hasher.Write(headerRaw)
	hasher.Write(payload)
	if !bytes.Equal(hasher.Sum(nil), frame.ChecksumSHA[:]) {
		return nil, NewMasError(ErrCodeWireSevered, "パケットチェックサム不一致", nil)
	}
	frame.Payload = payload
	return frame, nil
}

// ============================================================
// 2. 状態構造 & 構造化残差 (ResidualValue)
// ============================================================

type ConfidentValue struct {
	Value      string  `json:"value"`
	Confidence float64 `json:"confidence"`
	Entropy    float64 `json:"entropy"`
	Source     string  `json:"source"`
}

func (cv ConfidentValue) EffectiveConfidence() float64 {
	conf := clampValue(cv.Confidence, 0.0, 1.0)
	ent := clampValue(cv.Entropy, 0.0, 1.0)
	return math.Max(0.0, conf*(1.0-ent))
}

func (cv ConfidentValue) Clone() ConfidentValue {
	return ConfidentValue{
		Value:      cv.Value,
		Confidence: clampValue(cv.Confidence, 0.0, 1.0),
		Entropy:    clampValue(cv.Entropy, 0.0, 1.0),
		Source:     cv.Source,
	}
}

// ResidualValue は棄却・漏洩した情報を構造化して決定論的に管理する
type ResidualValue struct {
	OriginalData ConfidentValue `json:"original_data"`
	PurgeReason  string         `json:"purge_reason"`
	EvictedAt    LogicalClock   `json:"evicted_at"`
}

func (rv ResidualValue) Clone() ResidualValue {
	return ResidualValue{
		OriginalData: rv.OriginalData.Clone(),
		PurgeReason:  rv.PurgeReason,
		EvictedAt:    rv.EvictedAt,
	}
}

type HashB struct {
	ConfirmedState map[string]ConfidentValue `json:"confirmed_state"`
	TentativeState map[string]ConfidentValue `json:"tentative_state"`
	ResidualState  map[string]ResidualValue  `json:"residual_state"` // 構造化残差
	Agreements     []string                  `json:"agreements"`
	Undecided      []string                  `json:"undecided"`
	Clock          LogicalClock              `json:"clock"`
	Sequence       uint64                    `json:"sequence"`
}

func NewHashB() HashB {
	return HashB{
		ConfirmedState: make(map[string]ConfidentValue),
		TentativeState: make(map[string]ConfidentValue),
		ResidualState:  make(map[string]ResidualValue),
		Agreements:     make([]string, 0),
		Undecided:      make([]string, 0),
	}
}

func (h HashB) Clone() HashB {
	confirmedCopy := make(map[string]ConfidentValue, len(h.ConfirmedState))
	for k, v := range h.ConfirmedState {
		confirmedCopy[k] = v.Clone()
	}
	tentativeCopy := make(map[string]ConfidentValue, len(h.TentativeState))
	for k, v := range h.TentativeState {
		tentativeCopy[k] = v.Clone()
	}
	residualCopy := make(map[string]ResidualValue, len(h.ResidualState))
	for k, v := range h.ResidualState {
		residualCopy[k] = v.Clone()
	}
	return HashB{
		ConfirmedState: confirmedCopy,
		TentativeState: tentativeCopy,
		ResidualState:  residualCopy,
		Agreements:     append([]string(nil), h.Agreements...),
		Undecided:      append([]string(nil), h.Undecided...),
		Clock:          h.Clock,
		Sequence:       h.Sequence,
	}
}

func (h HashB) ComputeWireHash() (string, error) {
	cloned := h.Clone()
	sort.Strings(cloned.Agreements)
	sort.Strings(cloned.Undecided)
	bytesPayload, err := CanonicalMarshal(cloned)
	if err != nil {
		return "", err
	}
	frame, err := EncodeWireFrame4141(cloned.Sequence, cloned.Clock, bytesPayload)
	if err != nil {
		return "", err
	}
	hash := sha256.Sum256(frame)
	return hex.EncodeToString(hash[:]), nil
}

type ThinkingStyle string

const (
	StyleMathematician ThinkingStyle = "MATHEMATICIAN"
	StylePhysicist     ThinkingStyle = "PHYSICIST"
	StyleObserver      ThinkingStyle = "OBSERVER"
)

type InferenceFrame struct {
	Role       ThinkingStyle             `json:"role"`
	ParentHash string                    `json:"parent_hash"`
	Content    map[string]ConfidentValue `json:"content"`
	Reasoning  string                    `json:"reasoning"`
	Confidence float64                   `json:"confidence"`
	Sequence   uint64                    `json:"sequence"`
	CostUSD    float64                   `json:"cost_usd"`
	IsFallback bool                      `json:"is_fallback,omitempty"`
}

type ObservationFrame struct {
	ExcellentParts     map[string]ConfidentValue `json:"excellent_parts"`
	Issues             map[string]ConfidentValue `json:"issues"`
	Summary            string                    `json:"summary"`
	Confidence         float64                   `json:"confidence"`
	ReconstructionLoss float64                   `json:"reconstruction_loss"`
	ResidualContext    map[string]ResidualValue  `json:"residual_context"`
	CostUSD            float64                   `json:"cost_usd"`
	IsFallback         bool                      `json:"is_fallback,omitempty"`
}

func PackInferenceFrame(frame InferenceFrame, clock LogicalClock) ([]byte, error) {
	frame.Confidence = clampValue(frame.Confidence, 0.0, 1.0)
	for k, v := range frame.Content {
		v.Confidence = clampValue(v.Confidence, 0.0, 1.0)
		v.Entropy = clampValue(v.Entropy, 0.0, 1.0)
		frame.Content[k] = v
	}
	data, err := CanonicalMarshal(frame)
	if err != nil {
		return nil, err
	}
	return EncodeWireFrame4141(frame.Sequence, clock, data)
}

func UnpackInferenceFrame(wireBytes []byte) (*InferenceFrame, error) {
	decoded, err := DecodeAndVerifyFrame4141(wireBytes)
	if err != nil {
		return nil, err
	}
	var frame InferenceFrame
	if err := json.Unmarshal(decoded.Payload, &frame); err != nil {
		return nil, err
	}
	frame.Confidence = clampValue(frame.Confidence, 0.0, 1.0)
	return &frame, nil
}

// ============================================================
// 3. 情報理論的 ReconstructionLoss 動的算出エンジン
// ============================================================

func CalculateReconstructionLoss(mathFrame, physFrame *InferenceFrame, currentHashB HashB) float64 {
	totalInputKeys := make(map[string]bool)
	var sumEntropy float64
	var keyCount float64
	hasFallback := false

	processFrame := func(f *InferenceFrame) {
		if f == nil {
			return
		}
		if f.IsFallback {
			hasFallback = true
		}
		for k, v := range f.Content {
			totalInputKeys[k] = true
			sumEntropy += v.Entropy
			keyCount++
		}
	}

	processFrame(mathFrame)
	processFrame(physFrame)

	if keyCount == 0 {
		return 0.50 // 情報ゼロ時のベース損失
	}

	// 1. エントロピー（曖昧さ）による損失項目 (0.0 - 0.4)
	avgEntropy := sumEntropy / keyCount
	entropyLoss := avgEntropy * 0.4

	// 2. 被覆率 (Coverage Ratio) による損失項目 (0.0 - 0.3)
	// 過去の Confirmed 状態に対してどれだけ新思考フレームが被覆できているか
	confirmedCovered := 0
	for k := range currentHashB.ConfirmedState {
		if totalInputKeys[k] {
			confirmedCovered++
		}
	}
	coverageRatio := 1.0
	if len(currentHashB.ConfirmedState) > 0 {
		coverageRatio = float64(confirmedCovered) / float64(len(currentHashB.ConfirmedState))
	}
	coverageLoss := (1.0 - coverageRatio) * 0.3

	// 3. プロトコル障害ペナルティ (0.3)
	fallbackPenalty := 0.0
	if hasFallback {
		fallbackPenalty = 0.30
	}

	totalLoss := entropyLoss + coverageLoss + fallbackPenalty
	return clampValue(totalLoss, 0.0, 1.0)
}

// ============================================================
// 4. エージェント & LLM Determinist Adapter
// ============================================================

type Agent interface {
	Name() string
	Infer(ctx context.Context, role ThinkingStyle, parentHash string, currentHashB HashB, seq uint64, clock LogicalClock, previousWire []byte) ([]byte, error)
	Observe(ctx context.Context, mathWire, physWire []byte, currentHashB HashB) (ObservationFrame, error)
}

// DeterministAgentAdapter は非決定論的 LLM 出力を決定論的 Runtime 境界へ固定・接続する
type DeterministAgentAdapter struct {
	innerAgent Agent
	seed       int64
	temperature float64
}

func NewDeterministAgentAdapter(inner Agent, seed int64) *DeterministAgentAdapter {
	return &DeterministAgentAdapter{
		innerAgent: inner,
		seed:       seed,
		temperature: 0.0, // 決定論的挙動の強制
	}
}

func (a *DeterministAgentAdapter) Name() string { return a.innerAgent.Name() }

func (a *DeterministAgentAdapter) Infer(ctx context.Context, role ThinkingStyle, parentHash string, currentHashB HashB, seq uint64, clock LogicalClock, previousWire []byte) ([]byte, error) {
	// 将来的に LLM API リクエスト時の seed/temperature 固定処理をここに隠蔽
	return a.innerAgent.Infer(ctx, role, parentHash, currentHashB, seq, clock, previousWire)
}

func (a *DeterministAgentAdapter) Observe(ctx context.Context, mathWire, physWire []byte, currentHashB HashB) (ObservationFrame, error) {
	return a.innerAgent.Observe(ctx, mathWire, physWire, currentHashB)
}

type RoleAssignment struct {
	Mathematician Agent
	Physicist     Agent
	Observer      Agent
	Round         int
}

type RoundRobinOrchestrator struct {
	agents [3]Agent
}

func NewRoundRobinOrchestrator(a, b, c Agent) *RoundRobinOrchestrator {
	return &RoundRobinOrchestrator{agents: [3]Agent{a, b, c}}
}

func (o *RoundRobinOrchestrator) Assign(round int) RoleAssignment {
	idx := round % 3
	return RoleAssignment{
		Mathematician: o.agents[idx],
		Physicist:     o.agents[(idx+1)%3],
		Observer:      o.agents[(idx+2)%3],
		Round:         round,
	}
}

// ============================================================
// 5. 純化 (Purify) & 残差再浮上 (Resurrection) ロジック
// ============================================================

type PurifyPolicy struct {
	HighThreshold float64
	LowThreshold  float64
}

type PurifyStats struct {
	PurgedCount      int
	PromotedCount    int
	ResurrectedCount int
}

func Purify(input HashB, reconstructionLoss float64, policy PurifyPolicy) (HashB, PurifyStats) {
	purified := NewHashB()
	purified.Clock = input.Clock
	purified.Sequence = input.Sequence

	// 残差状態の引き継ぎ
	for k, v := range input.ResidualState {
		purified.ResidualState[k] = v.Clone()
	}

	stats := PurifyStats{}
	allEntries := make(map[string]ConfidentValue)
	for k, v := range input.TentativeState {
		allEntries[k] = v.Clone()
	}
	for k, v := range input.ConfirmedState {
		allEntries[k] = v.Clone()
	}

	lossFactor := 1.0 - clampValue(reconstructionLoss, 0.0, 1.0)

	for key, item := range allEntries {
		effectiveConf := item.EffectiveConfidence() * lossFactor
		item.Confidence = clampValue(effectiveConf, 0.0, 1.0)

		// 残差からの再浮上 (Resurrection) チェック
		if _, inResidual := purified.ResidualState[key]; inResidual && effectiveConf >= policy.LowThreshold {
			delete(purified.ResidualState, key)
			stats.ResurrectedCount++
		}

		if effectiveConf >= policy.HighThreshold {
			purified.ConfirmedState[key] = item
			purified.Agreements = append(purified.Agreements, key)
			if _, wasTentative := input.TentativeState[key]; wasTentative {
				if _, wasConfirmed := input.ConfirmedState[key]; !wasConfirmed {
					stats.PromotedCount++
				}
			}
		} else if effectiveConf >= policy.LowThreshold {
			purified.TentativeState[key] = item
			purified.Undecided = append(purified.Undecided, key)
		} else {
			// 構造化残差 (ResidualValue) に退避
			purified.ResidualState[key] = ResidualValue{
				OriginalData: item.Clone(),
				PurgeReason:  fmt.Sprintf("LowEffectiveConf(%.3f)_Loss(%.3f)", effectiveConf, reconstructionLoss),
				EvictedAt:    input.Clock,
			}
			stats.PurgedCount++
		}
	}

	sort.Strings(purified.Agreements)
	sort.Strings(purified.Undecided)
	return purified, stats
}

type SimpleStateStore struct {
	mu       sync.Mutex
	hashB    HashB
	sequence atomic.Uint64
}

func NewSimpleStateStore() *SimpleStateStore { return &SimpleStateStore{hashB: NewHashB()} }

func (s *SimpleStateStore) NextSequence() uint64 { return s.sequence.Add(1) }
func (s *SimpleStateStore) GetSequence() uint64 { return s.sequence.Load() }

func (s *SimpleStateStore) GetHashB() HashB {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.hashB.Clone()
}

func (s *SimpleStateStore) SetHashB(hb HashB) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.hashB = hb.Clone()
}

// ============================================================
// 6. HomeostasisPurifier Runtime (決定論的 State Machine)
// ============================================================

type HomeostasisPurifier struct {
	store      *SimpleStateStore
	policy     PurifyPolicy
	basis      ResourceBasis
	roundLimit time.Duration
	maxCostUSD float64
	logger     *slog.Logger
}

func NewHomeostasisPurifier(
	store *SimpleStateStore,
	policy PurifyPolicy,
	basis ResourceBasis,
	roundLimit time.Duration,
	maxCostUSD float64,
	logger *slog.Logger,
) (*HomeostasisPurifier, error) {
	if err := basis.Validate(); err != nil {
		return nil, err
	}
	if logger == nil {
		logger = slog.Default()
	}
	return &HomeostasisPurifier{
		store:      store,
		policy:     policy,
		basis:      basis,
		roundLimit: roundLimit,
		maxCostUSD: maxCostUSD,
		logger:     logger,
	}, nil
}

func (p *HomeostasisPurifier) getBoundedTimeout(ctx context.Context, allocatedRatio float64) (context.Context, context.CancelFunc) {
	allocatedDuration := time.Duration(float64(p.roundLimit) * allocatedRatio)
	deadline, ok := ctx.Deadline()
	if !ok {
		return context.WithTimeout(ctx, allocatedDuration)
	}
	remaining := time.Until(deadline)
	if remaining < allocatedDuration {
		return context.WithTimeout(ctx, remaining)
	}
	return context.WithTimeout(ctx, allocatedDuration)
}

func makeFallbackInferenceWire(role ThinkingStyle, parentHash string, seq uint64, clock LogicalClock, reason string, currentHashB HashB) ([]byte, error) {
	inheritedContent := make(map[string]ConfidentValue)
	for k, v := range currentHashB.ConfirmedState {
		inheritedContent[k] = ConfidentValue{
			Value:      v.Value,
			Confidence: clampValue(v.Confidence*0.9, 0.0, 1.0),
			Entropy:    0.25,
			Source:     "FallbackInherited",
		}
	}
	fb := InferenceFrame{
		Role:       role,
		ParentHash: parentHash,
		Content:    inheritedContent,
		Reasoning:  "Resilient Recovery: " + reason,
		Confidence: 0.30,
		Sequence:   seq,
		CostUSD:    0.0,
		IsFallback: true,
	}

	wire, err := PackInferenceFrame(fb, clock)
	if err != nil {
		return nil, NewMasError(ErrCodeStateTransitionFailed, "フォールバックフレームのエンコード失敗", err)
	}
	return wire, nil
}

func (p *HomeostasisPurifier) ExecuteDiscussionRound(
	ctx context.Context,
	roles RoleAssignment,
	currentHashB HashB,
	roundClock LogicalClock,
) (HashB, error) {
	seq := p.store.NextSequence()
	parentHash, err := currentHashB.ComputeWireHash()
	if err != nil {
		parentHash = deterministicErrorHash(err)
		p.logger.Error("⚠️ 親ハッシュ計算失敗 -> 決定論的フォールバックハッシュ使用", slog.String("fallback", parentHash), slog.Any("error", err))
	}

	mathMaxCost := p.maxCostUSD * p.basis.MathematicianRatio()
	physMaxCost := p.maxCostUSD * p.basis.PhysicistRatio()
	obsMaxCost := p.maxCostUSD * p.basis.ObserverRatio()

	p.logger.Info("🔄 ラウンド開始 (AXIOM Deterministic Engine v2.7)",
		slog.Int("round", roles.Round),
		slog.Int64("logical_clock", int64(roundClock)),
		slog.String("parent_hash", shortHash(parentHash, 8)),
	)

	// --- 1. Mathematician Step ---
	ctxMath, cancelMath := p.getBoundedTimeout(ctx, p.basis.MathematicianRatio())
	mathWire, err := roles.Mathematician.Infer(ctxMath, StyleMathematician, parentHash, currentHashB, seq, roundClock, nil)
	cancelMath()
	if err != nil {
		p.logger.Warn("⚠️ Mathematician 障害 -> フォールバック適用", slog.Any("error", err))
		mathWire, err = makeFallbackInferenceWire(StyleMathematician, parentHash, seq, roundClock, "Timeout/Error", currentHashB)
		if err != nil {
			return currentHashB, err
		}
	} else {
		if mathFrame, errCheck := UnpackInferenceFrame(mathWire); errCheck == nil && mathFrame.CostUSD > mathMaxCost {
			p.logger.Warn("⚠️ Mathematician 予算超過 -> フォールバック適用")
			mathWire, err = makeFallbackInferenceWire(StyleMathematician, parentHash, seq, roundClock, "BudgetExceeded", currentHashB)
			if err != nil {
				return currentHashB, err
			}
		}
	}

	// --- 2. Physicist Step ---
	ctxPhys, cancelPhys := p.getBoundedTimeout(ctx, p.basis.PhysicistRatio())
	physWire, err := roles.Physicist.Infer(ctxPhys, StylePhysicist, parentHash, currentHashB, seq+1, roundClock, mathWire)
	cancelPhys()
	if err != nil {
		p.logger.Warn("⚠️ Physicist 障害 -> フォールバック適用", slog.Any("error", err))
		physWire, err = makeFallbackInferenceWire(StylePhysicist, parentHash, seq+1, roundClock, "Timeout/Error", currentHashB)
		if err != nil {
			return currentHashB, err
		}
	} else {
		if physFrame, errCheck := UnpackInferenceFrame(physWire); errCheck == nil && physFrame.CostUSD > physMaxCost {
			p.logger.Warn("⚠️ Physicist 予算超過 -> フォールバック適用")
			physWire, err = makeFallbackInferenceWire(StylePhysicist, parentHash, seq+1, roundClock, "BudgetExceeded", currentHashB)
			if err != nil {
				return currentHashB, err
			}
		}
	}

	// --- 3. Observer Step ---
	ctxObs, cancelObs := p.getBoundedTimeout(ctx, p.basis.ObserverRatio())
	observation, err := roles.Observer.Observe(ctxObs, mathWire, physWire, currentHashB)
	cancelObs()
	if err != nil || observation.CostUSD > obsMaxCost {
		p.logger.Warn("⚠️ Observer 障害 -> Passthrough Policy")
		return currentHashB, nil
	}

	// 状態更新 & 純化処理
	candidate := currentHashB.Clone()
	candidate.Clock = roundClock
	candidate.Sequence = seq + 2

	for k, v := range observation.ExcellentParts {
		v.Confidence = clampValue(v.Confidence*(1.0+p.basis.ObserverRatio()), 0.0, 1.0)
		candidate.TentativeState[k] = v
	}
	for k, v := range observation.Issues {
		v.Confidence = clampValue(v.Confidence*(1.0-p.basis.ObserverRatio()), 0.0, 1.0)
		candidate.TentativeState[k] = v
	}
	for k, v := range observation.ResidualContext {
		candidate.ResidualState[k] = v
	}

	purified, stats := Purify(candidate, observation.ReconstructionLoss, p.policy)
	p.logger.Info("✨ セッション純化完了 (動的 ReconstructionLoss 適用)",
		slog.Int("round", roles.Round),
		slog.Float64("dynamic_reconstruction_loss", observation.ReconstructionLoss),
		slog.Int("Promoted", stats.PromotedCount),
		slog.Int("ResurrectedFromResidual", stats.ResurrectedCount),
		slog.Int("PurgedToResidual", stats.PurgedCount),
		slog.Int("Confirmed Total", len(purified.ConfirmedState)),
	)
	return purified, nil
}

// ============================================================
// 7. 実装エージェント (動的損失計算の実装)
// ============================================================

type ConcreteAgent struct {
	name        string
	simulateErr bool
	logger      *slog.Logger
}

func (a *ConcreteAgent) Name() string { return a.name }

func (a *ConcreteAgent) Infer(ctx context.Context, role ThinkingStyle, parentHash string, currentHashB HashB, seq uint64, clock LogicalClock, previousWire []byte) ([]byte, error) {
	if a.simulateErr && role == StylePhysicist {
		return nil, NewMasError(ErrCodeWireSevered, "Physicist 通信障害シミュレーション", nil)
	}
	content := make(map[string]ConfidentValue)
	if role == StyleMathematician {
		content["math_logic"] = ConfidentValue{Value: "論理検証OK", Confidence: 0.92, Entropy: 0.04, Source: a.name}
	} else {
		content["phys_logic"] = ConfidentValue{Value: "物理制約OK", Confidence: 0.88, Entropy: 0.08, Source: a.name}
	}
	frame := InferenceFrame{
		Role:       role,
		ParentHash: parentHash,
		Content:    content,
		Confidence: 0.90,
		Sequence:   seq,
		CostUSD:    0.002,
	}
	return PackInferenceFrame(frame, clock)
}

func (a *ConcreteAgent) Observe(ctx context.Context, mathWire, physWire []byte, currentHashB HashB) (ObservationFrame, error) {
	mathFrame, _ := UnpackInferenceFrame(mathWire)
	physFrame, _ := UnpackInferenceFrame(physWire)

	// 動的 ReconstructionLoss の算出
	reconstructionLoss := CalculateReconstructionLoss(mathFrame, physFrame, currentHashB)

	excellent := make(map[string]ConfidentValue)
	issues := make(map[string]ConfidentValue)
	residualCtx := make(map[string]ResidualValue)

	if mathFrame != nil {
		for k, v := range mathFrame.Content {
			excellent[k] = v
		}
	}
	if physFrame != nil {
		for k, v := range physFrame.Content {
			issues[k] = v
		}
	}

	return ObservationFrame{
		ExcellentParts:     excellent,
		Issues:             issues,
		Summary:            "統合完了",
		Confidence:         0.90,
		ReconstructionLoss: reconstructionLoss,
		ResidualContext:    residualCtx,
		CostUSD:            0.003,
	}, nil
}

// ============================================================
// 8. Golden Vector Test (正常系 & 障害系 決定論検証)
// ============================================================

func RunGoldenVectorTest(logger *slog.Logger) error {
	logger.Info("🧪 ゴールデンベクターテスト開始 (v2.7 完全決定論性を検証)...")
	policy := PurifyPolicy{HighThreshold: 0.70, LowThreshold: 0.30}

	runSimulation := func(simulateErr bool) (string, error) {
		agentA := NewDeterministAgentAdapter(&ConcreteAgent{name: "Node-A", simulateErr: false, logger: logger}, 42)
		agentB := NewDeterministAgentAdapter(&ConcreteAgent{name: "Node-B", simulateErr: simulateErr, logger: logger}, 42)
		agentC := NewDeterministAgentAdapter(&ConcreteAgent{name: "Node-C", simulateErr: false, logger: logger}, 42)

		orchestrator := NewRoundRobinOrchestrator(agentA, agentB, agentC)
		store := NewSimpleStateStore()
		purifier, err := NewHomeostasisPurifier(store, policy, DefaultResourceBasis, 100*time.Millisecond, 0.012, logger)
		if err != nil {
			return "", err
		}
		currentHashB := store.GetHashB()
		for r := 0; r < 2; r++ {
			roles := orchestrator.Assign(r)
			logicalClock := LogicalClock(1000 + r)
			nextHashB, err := purifier.ExecuteDiscussionRound(context.Background(), roles, currentHashB, logicalClock)
			if err != nil {
				return "", err
			}
			currentHashB = nextHashB
		}
		return currentHashB.ComputeWireHash()
	}

	normHash1, err1 := runSimulation(false)
	normHash2, err2 := runSimulation(false)
	if err1 != nil || err2 != nil || normHash1 != normHash2 {
		return fmt.Errorf("正常系ゴールデンベクター不一致: (1: %s, 2: %s)", normHash1, normHash2)
	}
	logger.Info(" ├─ ✅ 正常系 Strict Canonical Runtime 決定性チェック合格", slog.String("wire_hash", normHash1))

	failHash1, errF1 := runSimulation(true)
	failHash2, errF2 := runSimulation(true)
	if errF1 != nil || errF2 != nil || failHash1 != failHash2 {
		return fmt.Errorf("障害系ゴールデンベクター不一致: (1: %s, 2: %s)", failHash1, failHash2)
	}
	logger.Info(" └─ ✅ 障害系 (動的損失＋残差退避) Runtime 決定性チェック合格", slog.String("wire_hash", failHash1))
	return nil
}

// ============================================================
// 9. メインエントリーポイント
// ============================================================

func main() {
	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))
	slog.SetDefault(logger)
	logger.Info("=== axiom-mas-go v2.7 Deterministic Latent Engine Edition ===")

	if err := RunGoldenVectorTest(logger); err != nil {
		logger.Error("ゴールデンベクター検証失敗", slog.Any("error", err))
		os.Exit(1)
	}

	agentA := NewDeterministAgentAdapter(&ConcreteAgent{name: "Node-A", simulateErr: false, logger: logger}, 42)
	agentB := NewDeterministAgentAdapter(&ConcreteAgent{name: "Node-B", simulateErr: true, logger: logger}, 42)
	agentC := NewDeterministAgentAdapter(&ConcreteAgent{name: "Node-C", simulateErr: false, logger: logger}, 42)

	orchestrator := NewRoundRobinOrchestrator(agentA, agentB, agentC)
	store := NewSimpleStateStore()
	policy := PurifyPolicy{HighThreshold: 0.70, LowThreshold: 0.30}

	purifier, err := NewHomeostasisPurifier(store, policy, DefaultResourceBasis, 100*time.Millisecond, 0.012, logger)
	if err != nil {
		logger.Error("Purifier 初期化失敗", slog.Any("error", err))
		return
	}

	currentHashB := store.GetHashB()
	for r := 0; r < 3; r++ {
		roles := orchestrator.Assign(r)
		roundClock := LogicalClock(1000 + r)
		nextHashB, err := purifier.ExecuteDiscussionRound(context.Background(), roles, currentHashB, roundClock)
		if err != nil {
			logger.Error("セッション異常終了", slog.Any("error", err))
			return
		}
		currentHashB = nextHashB
		store.SetHashB(currentHashB)
	}

	logger.Info("🏁 全ラウンド完走完了",
		slog.Int("最終 ConfirmedState 数", len(currentHashB.ConfirmedState)),
		slog.Int("最終 ResidualState 数", len(currentHashB.ResidualState)),
	)
}

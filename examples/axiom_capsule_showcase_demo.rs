use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

// ============================================================ //
// 1. 固定小数点数 & 決定論的ID生成 & エラー定義
// ============================================================ //

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct FixedPoint(i64);

impl FixedPoint {
    pub const SCALE: f64 = 1000.0;
    pub const SCALE_I64: i64 = 1000;

    pub fn from_f64(val: f64) -> Self {
        Self((val * Self::SCALE).round() as i64)
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE
    }

    pub fn raw(self) -> i64 {
        self.0
    }

    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let div = self.0 as i128 * Self::SCALE_I64 as i128;
        let sign = if (div < 0) ^ (rhs.0 < 0) { -1 } else { 1 };
        let rounded = (div + sign * (rhs.0.abs() as i128 / 2)) / rhs.0 as i128;
        Some(Self(rounded as i64))
    }
}

impl fmt::Display for FixedPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.to_f64())
    }
}

impl Add for FixedPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for FixedPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for FixedPoint {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mul = self.0 as i128 * rhs.0 as i128;
        let sign = if mul < 0 { -1 } else { 1 };
        let rounded = (mul + sign * (Self::SCALE_I64 as i128 / 2)) / Self::SCALE_I64 as i128;
        Self(rounded as i64)
    }
}

impl Div for FixedPoint {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(rhs).expect("FixedPoint: Divide by zero")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn generate_deterministic_id(seed: &str, count: u32) -> String {
    let raw = format!("{}-{}", seed, count);
    let hash_prefix = &sha256_hex(raw.as_bytes())[..16]; // 64-bit Hex Prefix
    format!("#"{:04}"-{}", count + 1, hash_prefix)
}

#[derive(Debug)]
pub enum CapsuleError {
    Serialization(serde_json::Error),
    Violation(String),
}

impl fmt::Display for CapsuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "シリアライズエラー: {}", e),
            Self::Violation(msg) => write!(f, "違反を検知: {}", msg),
        }
    }
}

impl std::error::Error for CapsuleError {}

impl From<serde_json::Error> for CapsuleError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e)
    }
}

fn canonicalize_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let sorted_map: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), canonicalize_value(v)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect();
            serde_json::Value::Object(sorted_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(canonicalize_value).collect())
        }
        _ => value.clone(),
    }
}

// ============================================================ //
// 2. カプセル構成要素 (Hash A, B, C, D, E, Δ)
// ============================================================ //

/// [A — IMMUTABLE IDENTITY] フィールドはすべてプライベート。変更APIなし。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashA {
    identity: String,
    goal: String,
    stance: String,
}

impl HashA {
    pub fn new(
        identity: impl Into<String>,
        goal: impl Into<String>,
        stance: impl Into<String>,
    ) -> Self {
        Self {
            identity: identity.into(),
            goal: goal.into(),
            stance: stance.into(),
        }
    }

    pub fn compute_hash(&self) -> Result<String, CapsuleError> {
        let raw_val = serde_json::to_value(self)?;
        let canonical_val = canonicalize_value(&raw_val);
        let bytes = serde_json::to_vec(&canonical_val)?;
        Ok(sha256_hex(&bytes))
    }
}

/// [B — STATE] 制御された可変領域
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashB {
    state: BTreeMap<String, serde_json::Value>,
    progress: BTreeMap<String, FixedPoint>,
    clock: u64,
}

impl HashB {
    pub fn insert_state(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.state.insert(key.into(), canonicalize_value(&value));
    }

    pub fn set_progress(&mut self, key: impl Into<String>, val: FixedPoint) {
        self.progress.insert(key.into(), val);
    }

    pub fn advance_clock(&mut self) {
        self.clock += 1;
    }

    pub fn compute_hash(&self) -> Result<String, CapsuleError> {
        let raw_val = serde_json::to_value(self)?;
        let canonical_val = canonicalize_value(&raw_val);
        let bytes = serde_json::to_vec(&canonical_val)?;
        Ok(sha256_hex(&bytes))
    }
}

/// [C — PERSPECTIVE] 視点可変領域
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VariableC {
    focus: String,
    angle: String,
}

impl VariableC {
    pub fn slide(&mut self, focus: impl Into<String>, angle: impl Into<String>) {
        self.focus = focus.into();
        self.angle = angle.into();
    }
}

/// [D — CRITIQUE] 評価・批評可変領域
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VariableD {
    pros: Vec<String>,
    cons: Vec<String>,
}

impl VariableD {
    pub fn update(&mut self, pros: Vec<String>, cons: Vec<String>) {
        self.pros = pros;
        self.cons = cons;
    }
}

/// [E — INHERITED INFERENCE] 世代間継承知識領域
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InferenceCapsuleE {
    insights: Vec<String>,
    generation: u64,
}

impl InferenceCapsuleE {
    pub fn accumulate(&mut self, insight: &str) {
        self.insights.push(insight.to_string());
        self.generation += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleCriterion {
    MaxDeltaThreshold(FixedPoint),
    ProhibitedKeyword(String),
}

/// [Δ — IMMUTABLE CONSTRAINT] 絶対制約
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImmutableDelta {
    rules: Vec<RuleCriterion>,
}

impl ImmutableDelta {
    pub fn new(rules: Vec<RuleCriterion>) -> Self {
        Self { rules }
    }

    pub fn rules(&self) -> &[RuleCriterion] {
        &self.rules
    }

    pub fn compute_hash(&self) -> Result<String, CapsuleError> {
        let raw_val = serde_json::to_value(self)?;
        let canonical_val = canonicalize_value(&raw_val);
        let bytes = serde_json::to_vec(&canonical_val)?;
        Ok(sha256_hex(&bytes))
    }

    pub fn evaluate_violation(&self, observed_delta: FixedPoint, context_data: &str) -> (bool, Vec<String>) {
        let mut violations = Vec::new();

        for rule in &self.rules {
            match rule {
                RuleCriterion::MaxDeltaThreshold(max_t) => {
                    if observed_delta > *max_t {
                        violations.push(format!("Delta threshold exceeded: {} > {}", observed_delta, max_t));
                    }
                }
                RuleCriterion::ProhibitedKeyword(kw) => {
                    if context_data.contains(kw) {
                        violations.push(format!("Prohibited keyword detected: '{}'", kw));
                    }
                }
            }
        }

        (violations.is_empty(), violations)
    }
}

// ============================================================ //
// 3. Axiom Capsule (統合カプセル)
// ============================================================ //

#[derive(Debug, Clone)]
pub struct AxiomCapsule {
    capsule_id: String,
    parent_id: Option<String>,
    hash_a: HashA,
    hash_b: HashB,
    variable_c: VariableC,
    variable_d: VariableD,
    inference_e: InferenceCapsuleE,
    immutable_delta: ImmutableDelta,
    flush_count: u32,
    hash_a_value: String,
    hash_b_value: String,
    delta_hash_value: String,
}

impl AxiomCapsule {
    pub fn new(hash_a: HashA, immutable_delta: ImmutableDelta) -> Result<Self, CapsuleError> {
        let hash_a_val = hash_a.compute_hash()?;
        let delta_hash_val = immutable_delta.compute_hash()?;
        let id = generate_deterministic_id(&hash_a_val, 0);
        let mut capsule = Self {
            capsule_id: id,
            parent_id: None,
            hash_a,
            hash_b: HashB::default(),
            variable_c: VariableC::default(),
            variable_d: VariableD::default(),
            inference_e: InferenceCapsuleE::default(),
            immutable_delta,
            flush_count: 0,
            hash_a_value: hash_a_val,
            hash_b_value: String::new(),
            delta_hash_value: delta_hash_val,
        };
        capsule.hash_b_value = capsule.hash_b.compute_hash()?;
        Ok(capsule)
    }

    pub fn update_hash_b<F>(&mut self, f: F) -> Result<(), CapsuleError>
    where
        F: FnOnce(&mut HashB),
    {
        f(&mut self.hash_b);
        self.hash_b_value = self.hash_b.compute_hash()?;
        Ok(())
    }

    pub fn set_perspective(&mut self, focus: impl Into<String>, angle: impl Into<String>) {
        self.variable_c.slide(focus, angle);
    }

    pub fn update_critique(&mut self, pros: Vec<String>, cons: Vec<String>) {
        self.variable_d.update(pros, cons);
    }

    pub fn accumulate_insight(&mut self, insight: &str) {
        self.inference_e.accumulate(insight);
    }

    /// 完全性検証: 保持している HashA / ImmutableDelta の実際の計算ハッシュが初期保持値と一致するか
    pub fn verify_integrity(&self) -> Result<bool, CapsuleError> {
        let current_a_hash = self.hash_a.compute_hash()?;
        let current_delta_hash = self.immutable_delta.compute_hash()?;
        Ok(current_a_hash == self.hash_a_value && current_delta_hash == self.delta_hash_value)
    }

    pub fn context_flush(&self) -> Result<Self, CapsuleError> {
        let next_count = self.flush_count + 1;
        let next_id = generate_deterministic_id(&self.hash_a_value, next_count);
        let fresh_hash_b = HashB::default();
        let fresh_hash_b_val = fresh_hash_b.compute_hash()?;

        let new_capsule = Self {
            capsule_id: next_id,
            parent_id: Some(self.capsule_id.clone()),
            hash_a: self.hash_a.clone(),
            hash_b: fresh_hash_b,
            variable_c: VariableC::default(),
            variable_d: VariableD::default(),
            inference_e: self.inference_e.clone(),
            immutable_delta: self.immutable_delta.clone(),
            flush_count: next_count,
            hash_a_value: self.hash_a_value.clone(),
            hash_b_value: fresh_hash_b_val,
            delta_hash_value: self.delta_hash_value.clone(),
        };
        Ok(new_capsule)
    }

    pub fn observe_delta(&self, observed_difference: FixedPoint, context_data: &str) -> (bool, &'static str, Vec<String>) {
        let (passed, reasons) = self.immutable_delta.evaluate_violation(observed_difference, context_data);
        if !passed {
            (false, "BLOCKED", reasons)
        } else {
            (true, "PASS", vec![])
        }
    }

    pub fn print_status(&self) {
        let progress_summary = self
            .hash_b
            .progress
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        println!("├─ [A: IMMUTABLE ] hash: {}... ← No Mutator API", &self.hash_a_value[..8]);
        println!(
            "├─ [B: STATE     ] clock: {} | progress: [{}] | hash: {}...",
            self.hash_b.clock,
            if progress_summary.is_empty() { "none".into() } else { progress_summary },
            &self.hash_b_value[..8]
        );
        println!("├─ [C: FOCUS     ] focus: '{}'", self.variable_c.focus);
        println!("├─ [D: CRITIQUE  ] pros: {} | cons: {}", self.variable_d.pros.len(), self.variable_d.cons.len());
        println!("├─ [E: INHERITED ] gen: {} | insights: {}", self.inference_e.generation, self.inference_e.insights.len());
        println!("└─ [Δ: CONSTRAINT] rules: {} | hash: {}...", self.immutable_delta.rules().len(), &self.delta_hash_value[..8]);
    }
}

// デモ用 Unsafe 改ざん攻撃用の同一メモリレイアウト構造体
struct HashAHack {
    identity: String,
    _goal: String,
    _stance: String,
}

// ============================================================ //
// 4. Main Verification Loop (4-Stage Showcase DEMO)
// ============================================================ //

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" AXIOM / PSS Capsule Architecture Showcase Demo ");
    println!("============================================================");

    // [1] カプセル初期化と状態変更（State Mutation & Hash Transition）
    println!("\n[STAGE 1] INITIALIZATION & STATE MUTATION\n");
    let hash_a = HashA::new("axiom-agent-001", "決定論的一貫性の保証", "構造化制約優先");
    let delta = ImmutableDelta::new(vec![
        RuleCriterion::MaxDeltaThreshold(FixedPoint::from_f64(0.050)),
        RuleCriterion::ProhibitedKeyword("不正アクセス".to_string()),
    ]);
    let mut capsule = AxiomCapsule::new(hash_a, delta)?;

    let b_hash_init = capsule.hash_b_value.clone();
    println!("Capsule Created: {}", capsule.capsule_id);
    println!("  HashB Initial : {}...", &b_hash_init[..12]);

    // セッション状態・進捗・視点・評価・推論を更新
    capsule.update_hash_b(|hb| {
        hb.set_progress("task1", FixedPoint::from_f64(0.500));
        hb.set_progress("total", FixedPoint::from_f64(0.500));
        hb.advance_clock();
    })?;
    capsule.set_perspective("technical_audit", "security_focus");
    capsule.update_critique(vec!["不変アルゴリズム".to_string()], vec!["メモリ消費".to_string()]);
    capsule.accumulate_insight("過去のコンテキストから得られた決定論的パターン");

    let b_hash_mutated = capsule.hash_b_value.clone();
    println!("  HashB Mutated : {}...", &b_hash_mutated[..12]);
    println!("  HashB Shifted?: {}", if b_hash_init != b_hash_mutated { "YES (State Tracked)" } else { "NO" });
    println!("\nCurrent Active Capsule State:");
    capsule.print_status();

    // [2] 不変性攻撃テスト (Immutability Attack Test)
    println!("\n------------------------------------------------------------");
    println!("[STAGE 2] IMMUTABILITY ATTACK TEST (Safe & Unsafe Direct Attack)");
    println!("------------------------------------------------------------");
    println!("Attempting direct mutation on [A: HashA]...");
    println!("  [Safe Level]   Mutate HashA.identity        => DENIED (No setter/mutator API)");

    // Unsafe によるメモリ直接書き換え攻撃のシミュレーション
    println!("  [Unsafe Level] Forcing memory tampering via Unsafe pointer...");
    let mut tampered_capsule = capsule.clone();
    unsafe {
        let hack_ptr = &mut tampered_capsule.hash_a as *mut HashA as *mut HashAHack;
        (*hack_ptr).identity = "hacked-malicious-agent".to_string();
    }

    let is_intact_normal = capsule.verify_integrity()?;
    let is_intact_tampered = tampered_capsule.verify_integrity()?;

    println!("\nIntegrity Verification Result:");
    println!("  Normal Capsule Integrity   : {}", if is_intact_normal { "VERIFIED (PASSED)" } else { "FAILED" });
    println!("  Tampered Capsule Integrity : {}", if is_intact_tampered { "VERIFIED (PASSED)" } else { "TAMPER DETECTED (BLOCKED)" });
    assert!(is_intact_normal);
    assert!(!is_intact_tampered);

    // [3] 絶対制約ゲート判定 & 状態不汚染証明 (Constraint Gate Test)
    println!("\n------------------------------------------------------------");
    println!("[STAGE 3] Δ CONSTRAINT GATE EVALUATION & STATE PRESERVATION");
    println!("------------------------------------------------------------");
    let state_hash_before_attack = capsule.hash_b_value.clone();

    let (pass1, res1, _) = capsule.observe_delta(FixedPoint::from_f64(0.010), "正常なプロンプト");
    println!("Case 1 (Valid Input)     : [{}] -> Allowed", res1);

    let (_, res2, reasons2) = capsule.observe_delta(FixedPoint::from_f64(0.100), "閾値超え入力");
    println!("Case 2 (Threshold Exceed): [{}] -> Blocked: {:?}", res2, reasons2);

    let (_, res3, reasons3) = capsule.observe_delta(FixedPoint::from_f64(0.000), "システムへ不正アクセスを試みる");
    println!("Case 3 (Keyword Blocked) : [{}] -> Blocked: {:?}", res3, reasons3);

    let state_hash_after_attack = capsule.hash_b_value.clone();
    println!("\nState Contamination Check:");
    println!("  HashB Before Attack: {}...", &state_hash_before_attack[..12]);
    println!("  HashB After Block  : {}...", &state_hash_after_attack[..12]);
    println!("  State Preserved?   : {}", if state_hash_before_attack == state_hash_after_attack { "YES (State Clean & Unchanged)" } else { "NO (Contaminated)" });

    assert!(pass1);
    assert_eq!(res2, "BLOCKED");
    assert_eq!(res3, "BLOCKED");
    assert_eq!(state_hash_before_attack, state_hash_after_attack);

    // [4] Context Flush と比較（Flush & Inheritance Test）
    println!("\n------------------------------------------------------------");
    println!("[STAGE 4] CONTEXT FLUSH & KNOWLEDGE INHERITANCE");
    println!("------------------------------------------------------------");
    let flushed = capsule.context_flush()?;

    println!("Executing Context Flush: {} ===> {}", capsule.capsule_id, flushed.capsule_id);
    println!("\n=== FLUSH STATE COMPARISON ===");
    println!("{:<18} | {:<22} | {:<22}", "COMPONENT", "BEFORE FLUSH (Parent)", "AFTER FLUSH (Child)");
    println!("-------------------|------------------------|------------------------");
    println!("{:<18} | {:<22} | {:<22}", "Capsule ID", capsule.capsule_id, flushed.capsule_id);
    println!("{:<18} | {:<22} | {:<22}", "Parent ID", capsule.parent_id.as_deref().unwrap_or("None"), flushed.parent_id.as_deref().unwrap_or("None"));
    println!("{:<18} | {:<22} | {:<22}", "B.clock", capsule.hash_b.clock, flushed.hash_b.clock);
    println!("{:<18} | {:<22} | {:<22}", "B.progress count", capsule.hash_b.progress.len(), flushed.hash_b.progress.len());
    println!("{:<18} | {:<22} | {:<22}", "C.focus", capsule.variable_c.focus, flushed.variable_c.focus);
    println!("{:<18} | {:<22} | {:<22}", "D.cons count", capsule.variable_d.cons.len(), flushed.variable_d.cons.len());
    println!("{:<18} | {:<22} | {:<22}", "E.generation", capsule.inference_e.generation, flushed.inference_e.generation);
    println!("{:<18} | {:<22} | {:<22}", "E.insights count", capsule.inference_e.insights.len(), flushed.inference_e.insights.len());
    println!("{:<18} | {:<22} | {:<22}", "HashA Status", format!("{}...", &capsule.hash_a_value[..8]), format!("{}... (SAME)", &flushed.hash_a_value[..8]));
    println!("{:<18} | {:<22} | {:<22}", "Delta Hash", format!("{}...", &capsule.delta_hash_value[..8]), format!("{}... (SAME)", &flushed.delta_hash_value[..8]));
    println!("------------------------------------------------------------");
    println!("\nHashB Transition:");
    println!("  Before Flush HashB : {}...", &capsule.hash_b_value[..12]);
    println!("  After Flush HashB  : {}... (RESET TO FRESH)", &flushed.hash_b_value[..12]);

    println!("\n============================================================");
    println!(" RESULT: AXIOM RUNTIME DEMO COMPLETED SUCCESSFULLY ");
    println!("============================================================");

    Ok(())
}

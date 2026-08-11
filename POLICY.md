# AXIOM Framework — POLICY

**Status**: Constitutional  
**Applies to**: Production · Prototype 1.0 · PLP-R · ACP / Capsule · Round Consensus · UPR / Runtime  
**Date**: 2026-08-11  

> 実験は忠実に実際行って

この文書は README より一段上の **憲法** である。  
実装・RFC・研究線・Runtime はすべて本ポリシーに従う。  
ポリシー変更は実装変更と同じ重みで扱い、版を残す。

---

## Core Principles（不変の原則）

| Principle | Meaning |
|-----------|---------|
| **History is Truth** | 履歴（承認済み Seal / Hash 鎖）が真実。後からの書き換えで真実を上書きしない。 |
| **Canonical State is Immutable** | 一度確定した Canonical State は変更しない。新しい状態は新しい Capsule / Seal として積む。 |
| **Difference is Observable** | 差異は常に観測可能である。DCK / Dual Hash / DifferenceMetrics で数値化する。 |
| **Projection is Replaceable** | Projector は交換可能。意味の正しさではなく、決定論的投影の契約を満たせばよい。 |
| **Framework before Model** | 特定 LLM / モデルより、プロトコルと枠組みを優先する。モデルは差し替え可能である。 |

---

## 1. Design Policy

1. **Deterministic First**  
2. **Canonical before Optimization**  
3. **Immutable Core**  
4. **Research separated from Production**  

---

## 2. Hash Policy

1. **Domain Separation required**  
2. **Canonical serialization only**  
3. **Golden Vector mandatory**  
4. **Dual Hash clarity** — HashA/raw · HashB/canonical; Raw not folded into canonical_hash  

---

## 3. Compatibility Policy

1. **Never break released versions**  
2. **New behavior requires version bump**  
3. **Golden vectors are immutable**  
4. **payload version ≠ implementation version**  

---

## 4. Security Policy

1. **No hidden mutable state**  
2. **Hash verification required**  
3. **Observer isolation**  
4. **No silent history rewrite**  

---

## 5. Runtime Policy

1. **Projection must not modify Canonical State**  
2. **Monitor may request review but never rewrite history**  
3. **Round Reset keeps only approved snapshots**  
4. **Framework before Model**  

---

## 6. Testing Policy

1. **Every protocol change requires Golden tests**  
2. **Cross-language verification required**  
3. **Deterministic replay required**  
4. **Conformance over anecdote**  

---

## 7. Documentation Policy

1. **RFC before implementation**  
2. **Production and Research are separated**  
3. **Policies live in version control**  
4. **Design rationale is written down**  

---

## Enforcement

```
POLICY  >  released Golden / payload contract  >  ROADMAP  >  実装の都合
```

---

*History is Truth · Canonical is Immutable · Difference is Observable · Projection is Replaceable · Framework before Model*

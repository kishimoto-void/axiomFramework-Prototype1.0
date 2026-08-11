# Prototype 1.0 — Specification

**Milestone**: Difference Convergence Observation  
**Status**: Design-locked / implementation in progress  

> 実験は忠実に実際行って

---

## Goal

Deliver a runnable path:

```
Input → PSS → PLP → Capsule → ACP → DCK → Difference Report
```

Prototype 1.0 is complete when this path is:

1. **Deterministic** (same input → same hashes under a fixed payload version)
2. **Observable** (DCK always emits DualHashClass + DifferenceMetrics)
3. **History-safe** (no silent rewrite of sealed states)
4. **Documented** (Golden vectors + Difference Report schema)

---

## Non-goals

| Out of scope | Why |
|--------------|-----|
| Majority-vote multi-LLM | Not the research question |
| Semantic correctness of annotations | PLP does not claim meaning |
| Guaranteed monotonic convergence | Design hypothesis only |
| Production PLP v1.1.3 replacement | Research / Prototype line is parallel |
| Full Round Consensus / personality runtime | After Prototype 1.0 |
| Vendor LLM adapters | Framework before Model |

---

## Module contracts (summary)

| Module | Input | Output | Must not |
|--------|-------|--------|----------|
| **PSS** | raw bytes/text | NormalizedInput | invent semantics |
| **PLP** | NormalizedInput | Canonical State (+ tokens, optional candidate annotations) | claim Semantic Truth |
| **Capsule** | Raw + Canonical | sealed storage with Dual Hash | mutate prior capsules |
| **ACP** | Capsule + HashA contract | Seal / proof | rank answer quality |
| **DCK** | two sealed states | DualHashClass + DifferenceMetrics | rewrite history |
| **Runtime** | pipeline definition | ordered execution + Report | hide mutable side channels |

---

## Dual Hash

| Name | Source | Role |
|------|--------|------|
| **HashA** | raw / contract material | Invariant integrity |
| **HashB** | canonical payload | Projected state integrity |

Rules:

- Domain separation required (`axiom:v2:raw|canonical|proof`)
- Raw is **not** folded into HashB
- Payload version enters the hash; crate version does not

---

## Difference Report (required fields)

```text
DifferenceReport {
  left_id, right_id
  dual_hash_class     // None | Semantic | State | Compound
  metrics {
    overlap_ratio, divergence
    added, removed, changed
  }
  integrity_ok
  notes[]             // human-readable, non-authoritative
}
```

---

## Versioning

- **Payload version** enters the hash and Golden lock
- **Implementation (crate) version** does not enter the hash
- Breaking observation behavior ⇒ version bump + **new** Golden (never edit old Golden in place)

---

## Success checklist

- [ ] PSS normalizes sample EN/JA inputs
- [ ] PLP TokenOnly + Minimal project deterministically
- [ ] Capsule stores A/B + Dual Hash
- [ ] ACP produces verifiable seal
- [ ] DCK classifies and measures two capsules
- [ ] Runtime emits Difference Report for `data/sample_inputs/`
- [ ] At least one Golden vector LOCKED
- [ ] Divergence logged without claiming guaranteed shrink

---

*Difference Convergence Observation — that is Prototype 1.0.*

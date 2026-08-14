# ROADMAP — Prototype 1.0

**Milestone**: Difference Convergence Observation only

> 実験は忠実に実際行って

---

## Phase 0 — Scaffold (current)

- [x] Repository structure
- [x] README pipeline diagram
- [x] POLICY / architecture / prototype_spec / convergence docs
- [x] Workspace `Cargo.toml`
- [x] Crate stubs with public API outlines

## Phase 1 — PSS + PLP + PLP-R  ✅

- [x] PSS: normalize raw input (encoding, whitespace policy, language hint, CR strip)
- [x] PLP-R contracts in `axiom-plp`: Dual Hash, annotation candidates, payload `0.1.1`
- [x] TokenOnlyProjector (baseline) + MinimalProjector (demo)
- [x] Deterministic `build_canonical_payload` for Golden locks
- [x] Unit tests: axiom-pss 6/6 · axiom-plp 7/7
- [ ] Golden vector files under `tests/golden_vectors/` (lock hashes next)

## Phase 1.5 — LRP Kernel (parallel) ✅ 2026-08-14

- [x] `axiom-lrp` crate (v2.0.0-rfc-kernel)
- [x] IEEE 754 bit-exact StateHasher
- [x] Merkle chain + version-aware RuntimeHash
- [x] TransitionBuilder + ReasoningIntent
- [x] CapabilityResolver + RuntimePolicy
- [x] Plugin framework (Validator / Observer)
- [x] Chain + Snapshot verification
- [x] Unit tests (hash / builder / chain)

> LRP is **not** on the Prototype 1.0 core path (PSS→…→DCK).  
> It is the deterministic reasoning session kernel for later multi-agent / Round Consensus work.

## Phase 2 — Capsule + ACP

- [x] Capsule A/B layers (Raw + Canonical + Dual Hash) — basic `Capsule` present
- [ ] ACP seal: HashA contract + HashB projected + proof
- [ ] Domain-separated hashing
- [ ] Golden vectors including seals

## Phase 3 — DCK + Difference Report  ✅ taxonomy 2026-08-14

- [x] DualHashClass (None / Semantic / State / Compound) — **official v2.3**
- [x] DifferenceKind + ConstraintVerdict (Constraint priority)
- [x] DifferenceMetrics (overlap, divergence, added/removed)
- [x] Difference Report schema (`report` / `report_with_constraint`)
- [x] Capsule bridge: `classify` / `metrics` / `evaluate_capsules`
- [ ] Convergence experiment harness (log divergence; no guarantee of shrink)
- [ ] Golden vector files for DCK matrix

## Phase 4 — Runtime wire-up

- [ ] Linear pipeline: PSS → PLP → Capsule → ACP → DCK → Report
- [ ] `examples/minimal_pipeline`
- [ ] `examples/convergence_demo`
- [ ] Integration tests

## Explicitly deferred (after Prototype 1.0)

- Multi-agent Round Consensus (Observer rotation)
- Personality / emotional runtime
- Production PLP v2 promotion
- Full cross-language conformance CI
- LLM Provider adapters
- LRP ↔ Capsule / ACP tight integration

---

## Build order (do not reorder without reason)

```
PSS → PLP → Capsule → ACP → DCK → Runtime
```

This order keeps **Input → Difference Report** as one readable story.

LRP lives alongside and does not reorder the above.

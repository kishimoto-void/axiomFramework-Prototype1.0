# ROADMAP — Prototype 1.0

**Milestone**: Difference Convergence Observation only

> 実験は忠実に実際行って

---

## Phase 0 — Scaffold

- [x] Repository structure
- [x] README pipeline diagram
- [x] POLICY / architecture / prototype_spec / convergence docs
- [x] Workspace `Cargo.toml`

## Phase 1 — PSS + PLP + PLP-R  ✅

- [x] PSS: normalize raw input
- [x] PLP-R contracts in `axiom-plp` (payload `0.1.1`, research `0.1.2`)
- [x] TokenOnly + Minimal projectors
- [x] Deterministic `build_canonical_payload`
- [x] **diff.rs** — Canonical DifferenceMetrics (migrated from Axiom-Framework PLP-R)
- [x] **monitor.rs** — Continue / AskUser / Abort
- [x] `docs/plp-r/` research notes migrated
- [x] `tests/golden_vectors/PLP_R_GOLDEN_LOCK_v0_1.json`

## Phase 1.5 — LRP Kernel (parallel) ✅

- [x] `axiom-lrp` crate (alongside core path)

## Phase 2 — Capsule + ACP

- [x] Capsule A/B basic
- [ ] ACP seal + domain-separated proof
- [ ] Golden seals

## Phase 3 — DCK + Difference Report  ✅ taxonomy

- [x] Official DCK v2.3 dual-hash taxonomy in `axiom-dck`
- [x] Capsule bridge report API
- [ ] Convergence experiment harness
- [ ] DCK Golden matrix files

## Phase 4 — Runtime wire-up

- [ ] PSS → PLP → Capsule → ACP → DCK → Report
- [ ] examples + integration tests

## Explicitly deferred

- Round Consensus multi-agent runtime
- Production PLP v1.1.3 replacement (parallel only)
- Cross-language Golden CI automation

---

## Build order

```
PSS → PLP → Capsule → ACP → DCK → Runtime
```

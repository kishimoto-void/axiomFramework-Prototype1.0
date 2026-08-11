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

## Phase 1 — PSS + PLP

- [ ] PSS: normalize raw input (encoding, whitespace policy, language hint)
- [ ] PLP: TokenOnlyProjector (baseline) + MinimalProjector (demo)
- [ ] Golden vectors for projection hashes

## Phase 2 — Capsule + ACP

- [ ] Capsule A/B layers (Raw + Canonical + Dual Hash)
- [ ] ACP seal: HashA contract + HashB projected + proof
- [ ] Domain-separated hashing
- [ ] Golden vectors including seals

## Phase 3 — DCK + Difference Report

- [ ] DualHashClass (None / Semantic / State / Compound)
- [ ] DifferenceMetrics (overlap, divergence, added/removed)
- [ ] Difference Report schema (machine-readable + human summary)
- [ ] Convergence experiment harness (log divergence; no guarantee of shrink)

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

---

## Build order (do not reorder without reason)

```
PSS → PLP → Capsule → ACP → DCK → Runtime
```

This order keeps **Input → Difference Report** as one readable story.

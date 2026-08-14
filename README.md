# AXIOM Framework — Prototype 1.0

**Milestone**: Difference Convergence Observation  
**Status**: Phase 1 complete (PSS + PLP + PLP-R) + LRP Kernel v2.0.0-rfc-kernel  
**Date**: 2026-08-14  
**License**: [AXIOM Framework Research License v1.0](./LICENSE) (DCK: MIT)

> 実験は忠実に実際行って

---

## What this is

**Minimal configuration to demonstrate the core idea of AXIOM.**

Not a multi-agent showcase. Not a model zoo.  
Prototype 1.0 completes **one thing**:

> Observe whether differences between projected states can be measured, reported, and (as a design hypothesis) driven toward convergence under an immutable contract.

```
Input
  │
  ▼
PSS          (normalize input)
  │
  ▼
PLP          (project → Canonical State)
  │
  ▼
Capsule      (store Raw + Canonical + Dual Hash)
  │
  ▼
ACP          (seal under immutable contract / HashA)
  │
  ▼
DCK          (measure difference)
  │
  ▼
Difference Report
```

**Parallel component**: `axiom-lrp` (LRP Kernel v2.0.0-rfc-kernel) — deterministic reasoning session runtime (Merkle chain, TransitionBuilder, CapabilityResolver, Plugin framework). Not on the core Difference Observation path above.

---

## License

| Component | License |
|-----------|---------|
| Repository (PSS, PLP, Capsule, ACP, Runtime, LRP, docs) | **AXIOM Framework Research License v1.0** — see [`LICENSE`](./LICENSE) |
| `axiom-dck` (Difference Convergence Kernel) | **MIT** — see [`crates/dck/LICENSE-MIT`](./crates/dck/LICENSE-MIT) |

Research License allows personal, academic, educational, and non-commercial use with attribution.  
**Military / harm-oriented / commercial use** requires separate permission (commercial needs a contract).

---

## Priority order (build in this sequence)

| # | Crate | Role |
|---|-------|------|
| 1 | **pss** | Input normalization |
| 2 | **plp** | State projection (not meaning parsing) |
| 3 | **capsule** | State storage (A immutable / B projected) |
| 4 | **acp** | Immutable contract + seal / coordinates |
| 5 | **dck** | Difference observation (MIT) |
| 6 | **runtime** | Wire the pipeline end-to-end |
| — | **lrp** | Parallel: Deterministic LLM Reasoning Kernel (v2.0.0-rfc-kernel) |

Research lines (PLP-R, Round Consensus, etc.) live in the main Axiom-Framework repo.  
This Prototype reuses their contracts without mixing experiment logs into the core path.

---

## Core principles (from POLICY)

| Principle | Meaning here |
|-----------|----------------|
| History is Truth | Sealed states are not rewritten |
| Canonical is Immutable | New state ⇒ new Capsule / Seal |
| Difference is Observable | DCK always emits measurable metrics |
| Projection is Replaceable | Projectors are swappable |
| Framework before Model | No vendor LLM required for the core loop |

See `POLICY.md` (shared constitutional rules) and `docs/architecture.md`.

---

## Repository layout

```
axiomFramework-Prototype1.0/
├── README.md
├── LICENSE                 # Research License v1.0
├── ROADMAP.md
├── CHANGELOG.md
├── POLICY.md
├── docs/
├── crates/
│   ├── pss/ plp/ capsule/ acp/ runtime/   # Research License
│   ├── lrp/                               # LRP Kernel v2.0.0-rfc-kernel
│   └── dck/                               # MIT
├── examples/
├── tests/
├── data/sample_inputs/
└── tools/
```

---

## Success criterion (Prototype 1.0)

1. Same input → same Dual Hash across at least two implementations (or locked Golden).
2. Two Capsules → DCK emits `DualHashClass` + `DifferenceMetrics`.
3. Pipeline runs: **Input → Difference Report** with no silent mutation of history.
4. Divergence is **logged**, not claimed to always shrink (design hypothesis).

Multi-agent personality runtime, full Round Consensus, and production PLP v2 are **out of scope** for this milestone.

---

## Related

- Main research / production: [Axiom-Framework](https://github.com/kishimoto-void/Axiom-Framework)
- PLP-R results: `docs/plp-r/` in Axiom-Framework
- Round Consensus concept: `docs/ROUND_CONSENSUS_PROTOCOL_v0_1.md`

---

*Difference Convergence Observation — that is Prototype 1.0.*

# AXIOM Framework — Prototype 1.0 (Home)

**Milestone**: Difference Convergence Observation  
**Status**: Phase 1 + LRP + PLP Capsule + **ACP v1.2.0** + PSS Spec  
**Date**: 2026-08-14  
**License**: Research License (core) · MIT (PSS / DCK)

> 実験は忠実に実際行って

本リポジトリは **AXIOM の本家（home）** として運用します。

---

## What this is

**Minimal configuration to demonstrate the core idea of AXIOM.**

Prototype 1.0 completes **one thing**:

> Observe whether differences between projected states can be measured, reported, and (as a design hypothesis) driven toward convergence under an immutable contract.

```
Input
  │
  ▼
PSS          (normalize input)           — MIT
  │
  ▼
PLP          (project → Canonical State) — Research
  │
  ▼
Capsule      (store Raw + Canonical)     — Research
  │
  ▼
ACP          (seal / Frame / coordinate) — Research (v1.2.0)
  │
  ▼
DCK          (measure difference)        — MIT
  │
  ▼
Difference Report
```

**Parallel**: `axiom-lrp` (LRP Kernel) · `axiom-plp-capsule` · `axiom-pss-spec`

---

## License

| Component | Crate | License |
|-----------|-------|---------|
| **PSS** (normalize + Problem Specification) | `axiom-pss` / `axiom-pss-spec` | **MIT** |
| **DCK** (Difference Convergence Kernel) | `axiom-dck` | **MIT** |
| **ACP** (Common Protocol v1.2.0) | `axiom-acp` | **Research License** |
| **PLP** / PLP Capsule | `axiom-plp` / `axiom-plp-capsule` | **Research License** |
| **Capsule** | `axiom-capsule` | **Research License** |
| **LRP** | `axiom-lrp` | **Research License** |
| **Runtime** | `axiom-runtime` | **Research License** |

- Research License: 個人・学術・教育・非営利可。軍事・危害目的・商用は別途許諾が必要。
- 詳細: [`LICENSE`](./LICENSE) · MIT 各 crate の `LICENSE-MIT`

---

## Priority order

| # | Crate | Role | License |
|---|-------|------|---------|
| 1 | **pss** | Input normalization | MIT |
| 2 | **plp** | State projection | Research |
| 3 | **capsule** | A/B dual-hash storage | Research |
| 4 | **acp** | Seal + Frame / coordinate (v1.2.0) | Research |
| 5 | **dck** | Difference observation | MIT |
| 6 | **runtime** | Pipeline wire-up | Research |
| — | **lrp** | Reasoning kernel | Research |
| — | **pss-spec** | Full ProblemSpecification | MIT |
| — | **plp-capsule** | Production PLP Capsule v1.3.0 | Research |

---

## Core principles (from POLICY)

| Principle | Meaning |
|-----------|---------|
| History is Truth | Sealed states are not rewritten |
| Canonical is Immutable | New state ⇒ new Capsule / Seal |
| Difference is Observable | DCK always emits measurable metrics |
| Projection is Replaceable | Projectors are swappable |
| Framework before Model | No vendor LLM required for the core loop |

---

## Repository layout

```
axiomFramework-Prototype1.0/
├── LICENSE                 # Research License v1.0 (+ §7 MIT components)
├── crates/
│   ├── pss/ pss-spec/      # MIT
│   ├── dck/                # MIT
│   ├── plp/ plp-capsule/   # Research
│   ├── capsule/            # Research
│   ├── acp/                # Research (v1.2.0 production)
│   ├── lrp/                # Research
│   └── runtime/            # Research
├── docs/ examples/ tests/
└── …
```

---

## Related

- Legacy research mirror: [Axiom-Framework](https://github.com/kishimoto-void/Axiom-Framework)

---

*Difference Convergence Observation — Prototype 1.0 is home.*

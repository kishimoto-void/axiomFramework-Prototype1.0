# AXIOM Framework — Prototype 1.0 (Home)

**Milestone**: Difference Convergence Observation  
**Status**: **1.0.0** — CTS Full PASS  
**Date**: 2026-08-14  
**License**: Research License (core) · MIT (PSS / DCK)

> 実験は忠実に実際行って

本リポジトリは **AXIOM の本家（home）** として運用します。

---

## Pipeline

```
Input → PSS → PLP → Capsule → ACP → DCK → Difference Report
```

| Crate | Role | License |
|-------|------|---------|
| pss / pss-spec | Normalize + Problem Spec | MIT |
| plp | State projection (PLP-R) | Research |
| capsule | A/B dual-hash storage | Research |
| acp | Seal / Frame / coordinate v1.2.0 | Research |
| dck | Difference taxonomy v2.3 | MIT |
| runtime | Wire-up | Research |

---

## Conformance (CTS v1.0.0)

```bash
cargo test -p axiom-plp --test cts_v1
```

- Spec: [`CONFORMANCE.md`](./CONFORMANCE.md) · [`VERSIONING.md`](./VERSIONING.md)
- Levels: **Core** / **Full** / Experimental
- Latest report: **Full — 14/14 PASS**

CI pass requires: all Full tests · Golden Lock · Determinism ×100 · baseline divergence 0.

---

## Principles (POLICY)

| Principle | Meaning |
|-----------|---------|
| History is Truth | Sealed states are not rewritten |
| Canonical is Immutable | New state ⇒ new Capsule / Seal |
| Difference is Observable | DCK always emits measurable metrics |
| Projection is Replaceable | Projectors are swappable |
| Framework before Model | No vendor LLM required for the core loop |

---

## Related

- Legacy research mirror: [Axiom-Framework](https://github.com/kishimoto-void/Axiom-Framework)

*Difference Convergence Observation — Prototype 1.0 is home.*

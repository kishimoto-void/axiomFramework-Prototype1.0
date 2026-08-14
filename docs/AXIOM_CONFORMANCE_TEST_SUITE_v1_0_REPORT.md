# AXIOM Conformance Test Suite v1.0 — Execution Report

**Date**: 2026-08-14  
**Target**: axiomFramework-Prototype1.0  
**Runner**: `cargo test -p axiom-plp --test cts_v1`  
**Result**: **13 / 13 PASS**

---

## Suite definition

| No | Test | Pillar | Result |
|----|------|--------|--------|
| 01 | Golden Hash | Determinism | PASS |
| 02 | Golden Vector | Golden | PASS |
| 03 | Golden Coordinate | ACP domain coord | PASS |
| 04 | Cross-Language Hash | Compat (Golden lock) | PASS |
| 05 | Cross-Language Vector | Compat (Golden lock) | PASS |
| 06 | Canonical Serialization | Determinism | PASS |
| 07 | Hash Stability | Determinism | PASS |
| 08 | Difference Baseline | DCK / diff | PASS |
| 09 | Difference Threshold | DCK / Monitor | PASS |
| 10 | Difference Large Change | DCK / diff | PASS |
| 11 | Determinism Stress (×100) | Determinism | PASS |
| 12 | Regression Compatibility | Regression | PASS |
| 13 | End-to-End Pipeline | E2E PSS→…→DCK | PASS |

---

## Coverage

- Determinism (01, 06, 07, 11)
- Cross-language contract via Golden lock (04, 05, 12)
- Golden hash / vector / coordinate (01–03)
- Difference evaluation + Monitor (08–10)
- Regression (12)
- E2E Input → PSS → PLP → seal → difference (13)

## How to run

```bash
cargo test -p axiom-plp --test cts_v1
```

Golden lock: `tests/golden_vectors/PLP_R_GOLDEN_LOCK_v0_1.json`

---

*実験は忠実に実際行って*

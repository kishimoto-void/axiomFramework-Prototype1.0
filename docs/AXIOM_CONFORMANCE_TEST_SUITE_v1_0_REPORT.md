# AXIOM Conformance Test Suite v1.0.0 — Execution Report

**Date**: 2026-08-14  
**CTS_VERSION**: `1.0.0`  
**Target**: axiomFramework-Prototype1.0  
**Runner**: `cargo test -p axiom-plp --test cts_v1`  
**Result**: **14 / 14 PASS** (13 functional + 1 metadata)

**Conformance level achieved**: **Full**

| No | Test | Level | Result |
|----|------|-------|--------|
| 00 | Version + Golden metadata | Core | PASS |
| 01–13 | Functional suite | Core/Full | PASS |

CI criteria: 13 functional PASS · Golden lock · Determinism ×100 · Baseline divergence 0

Tolerance: hashes exact; floats `absolute_tolerance = 1e-9`

*実験は忠実に実際行って*

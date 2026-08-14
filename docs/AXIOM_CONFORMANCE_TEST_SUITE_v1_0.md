# AXIOM Conformance Test Suite v1.0

Formal prototype test suite (13 tests).

## Pillars

1. **Determinism** — same input → same hashes / serialization
2. **Cross-language compatibility** — Golden lock shared with Python/Go
3. **Difference** — baseline / threshold / large change + Monitor

## Tests

| No | Name | Purpose |
|----|------|--------|
| 01 | Golden Hash | Same input → same dual hash |
| 02 | Golden Vector | Intermediate vector matches baseline |
| 03 | Golden Coordinate | Domain-separated coordinate stable |
| 04 | Cross-Language Hash | Golden lock digests well-formed |
| 05 | Cross-Language Vector | Golden lock vectors well-formed |
| 06 | Canonical Serialization | Payload bytes identical |
| 07 | Hash Stability | Independent of run environment |
| 08 | Difference Baseline | Identical → divergence 0 |
| 09 | Difference Threshold | Small change measurable |
| 10 | Difference Large Change | Large change observable |
| 11 | Determinism Stress | 100× identical results |
| 12 | Regression Compatibility | Golden set size + canary digests |
| 13 | End-to-End Pipeline | PSS→PLP→seal→difference |

## Run

```bash
cargo test -p axiom-plp --test cts_v1
```

Latest report: `docs/AXIOM_CONFORMANCE_TEST_SUITE_v1_0_REPORT.md`

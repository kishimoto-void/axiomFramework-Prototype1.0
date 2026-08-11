# Phase 1 — PSS + PLP + PLP-R

**Date**: 2026-08-11  
**Status**: ✅ Implemented (unit tests PASS)  
**Repo**: axiomFramework-Prototype1.0  

> 実験は忠実に実際行って

---

## What shipped

| Crate | Role |
|-------|------|
| `axiom-pss` | Input normalization |
| `axiom-plp` | State Projection with **PLP-R contracts** |

```
raw text
  → PSS.normalize
  → NormalizedInput
  → PLP project_token_only | project_minimal
  → Projection { header, raw, canonical, raw_hash, canonical_hash }
```

---

## PLP-R contracts honored

1. PLP does not parse meaning  
2. Annotation = Canonical Projection Candidate  
3. Dual Hash (HashA=raw, HashB=canonical)  
4. TokenOnly / Minimal projector hierarchy  
5. Deterministic serialization for Golden  
6. payload version `0.1.1` ≠ crate version  

---

## Tests

```bash
cargo test -p axiom-pss -p axiom-plp
# axiom-pss: 6 passed
# axiom-plp: 7 passed
```

---

## Next (Phase 2)

- Capsule A/B storage crate wiring
- ACP seal + domain-separated proof
- Golden vector files locked under `tests/golden_vectors/`

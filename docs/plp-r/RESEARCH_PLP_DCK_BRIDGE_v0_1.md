# PLP-R Phase 2 — DCK Bridge Research Note

**Status**: ✅ Bridge demo PASS  
**Mapping**: PLP Dual Hash → DCK DualHashClass taxonomy

---

## Mapping

| PLP field | DCK role |
|-----------|----------|
| `raw_hash` | HashA (Invariant / physical) |
| `canonical_hash` | HashB (Semantic / projected) |

| A same | B same | DualHashClass |
|--------|--------|---------------|
| yes | yes | None |
| yes | no | Semantic |
| no | yes | State |
| no | no | Compound |

---

## Flow

```
Projection A / Projection B
        ↓
  evaluate dual hash + DifferenceMetrics
        ↓
  DCK classify / report
        ↓
  Monitor (optional)
```

Canonical annotation divergence is the primary continuous signal.  
Header-only noise must not drive AskUser.

---

*実験は忠実に実際行って*

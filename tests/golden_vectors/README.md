# Golden vectors

Once LOCKED for a payload version, do not rewrite in place.

## Contents

| File | Role |
|------|------|
| `PLP_R_GOLDEN_LOCK_v0_1.json` | PLP-R Phase 1 dual-hash lock (CTS v1.0) |
| `AXIOM_PROTOTYPE_GOLDEN_VECTORS_v1.0.md` | **13 observation points** for Prototype invariants |
| `baseline_identity.json` … `dck_monotonic_convergence.json` | Individual Golden Vectors (v1.0) |

See [`AXIOM_PROTOTYPE_GOLDEN_VECTORS_v1.0.md`](./AXIOM_PROTOTYPE_GOLDEN_VECTORS_v1.0.md) for the full rationale and table.

These 13 vectors fix the core invariants of the pipeline:

- Determinism & canonicalization (01–09)
- Hash A / Hash B separation (10–11)
- DCK zero-diff and monotonic convergence (12–13)

They are intentionally language-independent JSON so that Python / Rust / Go runners can share the same observation points.

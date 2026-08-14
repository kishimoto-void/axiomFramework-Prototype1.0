# Changelog

## [0.1.0] — Prototype 1.0

### LRP Kernel — 2026-08-14

#### LRP (`axiom-lrp`) v2.0.0-rfc-kernel
- Deterministic LLM Runtime Kernel (IEEE 754 bit-exact StateHasher)
- Merkle chained transitions (protocol-version-aware)
- Capability dependency resolver + RuntimePolicy evaluation
- TransitionBuilder (`&self` reusable) + ReasoningIntent
- Plugin system (ValidatorPlugin / ObserverPlugin) with panic recovery
- Snapshot modes (Full / Delta / Compressed) + full chain verification
- Unit tests: hash determinism, builder reusability, chain integrity

> Note: LRP is a parallel reasoning runtime. Prototype 1.0 core path (PSS → PLP → Capsule → ACP → DCK) is unchanged.

### Phase 1 — 2026-08-11

#### PSS (`axiom-pss`)
- `normalize` / `normalize_with_language`
- Trim ends, strip CR, reject empty
- Heuristic language hint (`ja` / `en`)

#### PLP + PLP-R (`axiom-plp`)
- Payload version `0.1.1`, protocol `PLP-R/0.1`
- Dual Hash: `raw_hash` (HashA) / `canonical_hash` (HashB)
- Deterministic `build_canonical_payload`
- `project_token_only` — annotations empty, `annotation_status=none`
- `project_minimal` — ACTION/ENTITY/LOCATION **candidates** only
- Unit tests: pss 6/6, plp 7/7 PASS

### Scaffold — 2026-08-11
- Workspace layout, POLICY, ROADMAP, docs

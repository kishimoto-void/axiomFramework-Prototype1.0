# Changelog

## [0.1.0] — Prototype 1.0

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

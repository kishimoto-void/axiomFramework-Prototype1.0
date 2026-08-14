# Changelog

## [1.0.0] — Conformance lock — 2026-08-14

### AXIOM Conformance Test Suite v1.0.0
- Formal CTS gate: 13 functional tests + metadata (`cts_00`)
- Levels: **Core** / **Full** / **Experimental**
- CI pass criteria fixed in `CONFORMANCE.md`
- Prototype CTS vs ACP Full CTS distinguished
- Tolerance: hashes exact; float `absolute_tolerance = 1e-9`
- Extension rule: never mutate tests 01–13

### Golden Lock metadata
- Wrapper: `schema_version`, `lock_version`, `cts_version`, `protocol_version`
- `hash_algorithm`, `generated_by`, `created_at`, `vectors[]`

### Release hygiene
- `VERSIONING.md` — SemVer + hash freeze policy
- `CONFORMANCE.md` — levels, CI, tolerances
- Execution: **14/14 PASS** (Full level)

## [0.1.0] — Prototype 1.0 (Home)

### ACP v1.2.0 + License policy — 2026-08-14

#### `axiom-acp` v1.2.0 (Research License)
- Stream JCS writer (RFC 8785) + domain-separated hash
- Prototype seal API retained (`contract_from_text` / `seal`)

#### License split
- **MIT**: `axiom-pss`, `axiom-pss-spec`, `axiom-dck`
- **Research License**: ACP, PLP, Capsule, LRP, Runtime, plp-capsule

### Phase 1 — PSS / PLP / PLP-R / DCK v2.3 taxonomy
- Dual hash projection, Monitor, Difference metrics

### Scaffold — 2026-08-11
- Workspace, POLICY, ROADMAP, docs

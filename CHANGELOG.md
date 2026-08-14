# Changelog

## [0.1.0] — Prototype 1.0 (Home)

### ACP v1.2.0 + License policy — 2026-08-14

#### `axiom-acp` v1.2.0 (Research License)
- Stream JCS writer (RFC 8785) + domain-separated hash (`[u8; 32]`)
- DomainTag const (`STATE` / `GENESIS` / `TRANSITION` / `PROOF` / `FRAME`)
- AxiomFrame / Genesis / TransitionRecord / ProofEnvelope / AxiomCore
- Causal chain verification + OnceLock transition hash cache
- `time` crate RFC 3339 normalize
- Prototype seal API retained (`contract_from_text` / `seal`)

#### License split (explicit)
- **MIT**: `axiom-pss`, `axiom-pss-spec`, `axiom-dck` (+ `LICENSE-MIT` per crate)
- **Research License**: ACP, PLP, Capsule, LRP, Runtime, plp-capsule
- Root `LICENSE` §7 / §8 updated

### PSS Spec — 2026-08-14

#### `axiom-pss-spec` v1.0.0-rc1 (MIT)
- Full Problem Specification Standard (ProblemSpecification + ProblemBuilder)
- Mission / SubMission / Knowledge / Constraints / Scope
- ThinkingProfile / PredictionPolicy / EvaluationCriteria
- Phase gates + ValidationReport + compile_for_generic

### PLP Capsule v1.3.0 — 2026-08-14

#### `axiom-plp-capsule` (Research License)
- Streaming canonical content hash + Observer / SchemaProvider
- Zero-alloc ObsKey + delta construction

### LRP Kernel — 2026-08-14

#### `axiom-lrp` v2.0.0-rfc-kernel (Research License)
- Deterministic LLM Runtime Kernel + Merkle chain + Plugin framework

### Phase 1 — 2026-08-11

#### PSS (`axiom-pss`) / PLP (`axiom-plp`)
- Input normalize + PLP-R dual hash projection

### Scaffold — 2026-08-11
- Workspace layout, POLICY, ROADMAP, docs

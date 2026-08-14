# VERSIONING — axiomFramework-Prototype1.0

## Product versions

| Component | Version | Notes |
|-----------|---------|-------|
| **Prototype product** | `1.0.0` | Difference Convergence Observation milestone |
| **CTS** | `1.0.0` | AXIOM Conformance Test Suite |
| **Golden Lock (PLP-R)** | `0.1.0` | payload `0.1.1`, research package `0.1.2` |
| **ACP** | `1.2.0` | Frame / JCS / domain tags |
| **DCK taxonomy** | `2.3` | Dual-hash classification |
| **PLP payload** | `0.1.1` | Hash-relevant serialization freeze |

## SemVer policy

- **MAJOR**: breaking change to public protocol hashes, Golden lock digests, or CTS required set
- **MINOR**: additive tests / optional metrics / new projectors (old Golden remains valid)
- **PATCH**: docs, CI, non-hash bugfixes

## CTS evolution rule

> Existing CTS v1.0 tests **must not be modified in place**.  
> New behaviour → new tests in CTS v1.1 / v2.0.

| CTS version | Golden Lock | Status |
|-------------|-------------|--------|
| `1.0.0` | PLP-R Golden Lock schema `1.0` / lock `0.1.0` | **Current** |

## Hash freezes

| Artifact | Frozen field | Algorithm |
|----------|--------------|-----------|
| PLP Dual Hash | `raw_hash`, `canonical_hash` | SHA-256 |
| ACP domain hash | DomainTag + RFC 8785 JCS | SHA-256 |
| ACP prototype seal | `axiom:v2:proof` material | SHA-256 |

Changing any frozen field requires a **MAJOR** bump and a new Golden generation.

---

*実験は忠実に実際行って*

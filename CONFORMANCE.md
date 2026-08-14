# CONFORMANCE — AXIOM Conformance Test Suite

**CTS_VERSION** = `1.0.0`  
**Product** = Prototype 1.0.0  
**Date locked** = 2026-08-14

---

## 1. Two suites (do not confuse)

| Suite | Scope | Runner | Dependencies |
|-------|--------|--------|--------------|
| **Prototype CTS v1.0** | PSS → PLP → Capsule fields → seal material → DCK metrics | `cargo test -p axiom-plp --test cts_v1` | No `time` crate required |
| **ACP Full CTS** | Frame / Genesis / JCS / causal chain / coordinate_id | `cargo test -p axiom-acp` | May require modern `time` / toolchain |

Prototype CTS is the **reference gate** for the Difference Convergence pipeline.  
ACP Full CTS is the **protocol implementation gate**.

---

## 2. Conformance Levels

| Level | Meaning | Required tests (CTS v1.0) |
|-------|---------|---------------------------|
| **Core** | Must pass for any claim of “AXIOM Prototype compatible” | 00, 01, 02, 06, 07, 08, 11, 12, 13 |
| **Full** | Recommended for reference implementations | Core + 03, 04, 05, 09, 10 |
| **Experimental** | Optional / future projectors, multi-agent | *(none in v1.0 — reserved)* |

Prototype 1.0 release target: **Full**.

---

## 3. CI success criteria (CTS pass)

A build is **CTS-PASS** only if **all** hold:

1. **All Core + Full tests PASS** on Prototype CTS v1.0 (`cts_v1`)
2. **Golden Lock** file present and `schema_version` / `cts_version` compatible
3. **Cross-language contract**: locked digests remain hex-64 and canary prefixes match
4. **Determinism**: stress test (×100) reports identical dual hashes
5. **Difference baseline**: identical projections → divergence == 0.0 (within abs tolerance)

Optional (ACP Full):

6. `cargo test -p axiom-acp` on a toolchain that supports ACP dependencies

---

## 4. Tolerance policy

| Quantity | Type | Tolerance |
|----------|------|-----------|
| SHA-256 digests | exact | **0** (bit-identical hex) |
| Canonical payload bytes | exact | **0** |
| DualHashClass / enum kinds | exact | equality |
| `overlap_ratio` / `divergence` | float | `absolute_tolerance = 1e-9` |
| Monitor threshold default | exact | `0.0` |

**Rule**: Hash and class comparisons never use tolerance.  
Float metrics use absolute tolerance only for non-rounded intermediates.

```text
absolute_tolerance = 1e-9
relative_tolerance = 0.0
```

---

## 5. Golden Lock correspondence

| CTS | Golden Lock file | lock_version | payload |
|-----|------------------|--------------|----------|
| 1.0.0 | `tests/golden_vectors/PLP_R_GOLDEN_LOCK_v0_1.json` | `0.1.0` | `0.1.1` |

---

## 6. Extension policy (v1.1+)

- Do **not** edit or renumber CTS 01–13
- Add new tests as 14+ under CTS_VERSION `1.1.0` or later
- New Golden vectors → new ids; never rewrite locked digests

---

## 7. How to run

```bash
cargo test -p axiom-plp --test cts_v1
cargo test -p axiom-acp   # Full ACP when toolchain allows
```

---

*実験は忠実に実際行って*

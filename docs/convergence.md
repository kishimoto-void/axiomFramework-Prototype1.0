# Difference Convergence — Prototype 1.0

**Status**: Design hypothesis (not a theorem)  

> 実験は忠実に実際行って

---

## What we mean

**Difference Convergence Observation** is the ability to:

1. Project inputs into comparable Canonical States
2. Seal them under an immutable contract (HashA)
3. Measure differences (HashB / annotations / metrics)
4. Log whether successive differences shrink, stay, or grow

Prototype 1.0 **observes**. It does **not** guarantee monotonic shrink.

---

## Why this is the milestone

```
Input → … → Difference Report
```

If this path is solid:

- Multi-agent Round Consensus can sit on top (Observer uses the same metrics)
- Personality / emotional runtimes can emit HashB without breaking history
- Production promotion has a clear conformance gate

If this path is weak, every higher layer invents its own ad-hoc “diff.”

---

## Measurement

| Signal | Producer | Use |
|--------|----------|-----|
| DualHashClass | DCK | None / Semantic / State / Compound |
| DifferenceMetrics | DCK | overlap, divergence, added/removed |
| integrity_ok | ACP verify | reject corrupted seals |
| divergence series | experiment harness | plot / compare policies |

Primary comparison surface: **Canonical annotations / projected state**  
Header-only noise must not drive “AskUser” or convergence claims.

---

## Hypothesis (testable)

> Holding HashA fixed and resetting ephemeral reasoning state between rounds  
> **may** reduce harmful bias accumulation and **may** improve measured convergence rates.

Prototype 1.0 must make this **falsifiable**:

- Log divergence per step
- Do not assert “always converges”
- Prefer tables and Golden locks over anecdotes

---

## Explicit non-claims

- Divergence does not always decrease
- Accept is not “best answer”
- Annotations are not ground-truth semantics

---

## Experiment slots (`docs/experiments/`)

Record each run with:

- input ids
- payload version
- DualHashClass sequence
- divergence series
- whether any seal failed verification

---

*Observe first. Claim convergence only with numbers.*

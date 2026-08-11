# Architecture — Prototype 1.0

## Pipeline

```
Input
  → PSS        normalize
  → PLP        project to Canonical State (no meaning claim)
  → Capsule    Raw (A) + Canonical (B) + Dual Hash
  → ACP        seal under HashA contract
  → DCK        classify + measure difference
  → Report     Difference Report
```

## Layer responsibilities

| Layer | Owns | Does not own |
|-------|------|--------------|
| PSS | encoding, whitespace, language hint | semantics |
| PLP | deterministic projection | truth / NLU |
| Capsule | storage of A/B + hashes | policy decisions |
| ACP | immutable contract + seal | quality ranking |
| DCK | DualHashClass + metrics | rewriting state |
| Runtime | ordering stages | model weights |

## Dual Hash

| Name | Alias | Role |
|------|-------|------|
| HashA | raw_hash | physical / contract integrity |
| HashB | canonical_hash | projected state integrity |

Domain separation required (`axiom:v2:raw|canonical|proof`).

## Difference Convergence (hypothesis)

DCK **observes** divergence.  
Prototype 1.0 does **not** guarantee monotonic shrink.  
Experiments may test whether resets + HashA anchoring improve convergence rates.

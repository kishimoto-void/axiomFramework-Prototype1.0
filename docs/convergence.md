# Difference Convergence — Design Note

## Claim boundary

> Under a fixed HashA (immutable contract), projecting inputs into Canonical States and measuring Dual Hash / annotation divergence makes difference **observable**. Whether repeated observation + reset **reduces** divergence is a **design hypothesis**, not a theorem of Prototype 1.0.

## What we measure

| Metric | Source |
|--------|--------|
| DualHashClass | None / Semantic / State / Compound |
| overlap_ratio | annotation set intersection / union |
| divergence | `1 - overlap_ratio` |
| added / removed | annotation keys |

## What we do not claim

- Divergence always decreases each step
- Semantic agreement between agents
- That Observer quality-ranking is required (it is forbidden)

## Why this is the Prototype 1.0 hero metric

Everything above DCK (multi-agent, personality, LLM adapters) needs a trustworthy difference signal.  
Prototype 1.0 finishes that signal first.

# axiomMAS

Impurity-Aware Multi-Agent System (v2.7)

## Files

- `axiom_mas_v27.go` : Go implementation of the Homeostasis Purifier MAS with Impurity Blacklist and Resource Penalty (v2.7).
- `axiom-mas-v27-experiment-zenn.md` : 10-round experiment report written for Zenn, including Grok analysis comparing modern MAS issues (topic shift / interference).

## Quick run (demo)

```bash
go run axiom_mas_v27.go
```

## Experiment summary

- 10 rounds with intentional fault injection on Node-B (Physicist role).
- Violations accumulated on rounds 0,3,6,9 → penalty factor went 1.0 → 0.95 → 0.85 → 0.70 → 0.05 (Out).
- See the Zenn MD for full analysis.

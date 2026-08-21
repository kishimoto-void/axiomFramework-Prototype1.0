# axiomMAS

Impurity-Aware Multi-Agent System (Go prototypes)

## Files

### v3.2.2 (current)
- `axiom_mas_v3.2.2_AbsoluteAnchor.go` — Dual Hash + **AbsoluteAnchor** (HashA as coordinate frame) + Human Gate
  - AbsoluteAnchor: problem / C / constraints / skeleton / seed → fixed HashA (time-independent)
  - Invariant constraint: `lawful:simulation_must_not_violate_law` (シミュレーションであっても法を犯さない)
  - 12-round protocol: provisional (R0–5) → R7 alignment → formal (R6–11)
  - Observer feasibility: possible / impossible / ask_human
  - Human decisions: adopt → Confirmed only; impossible cannot be adopted
- `MAS_Experiment_Report_v3.2.2.md` — Experiment report (knowledge-bot answer policy)

### v2.7 (legacy)
- `axiom_mas_v27.go` — Homeostasis Purifier with Impurity Blacklist / Resource Penalty
- `axiom-mas-v27-experiment-zenn.md` — 10-round experiment notes

## Quick run (v3.2.2)

```bash
go run axiom_mas_v3.2.2_AbsoluteAnchor.go
```

## v3.2.2 experiment highlights

- AbsoluteAnchor continuity across 12 capsules: same HashA
- R7 alignment: matched axes include `law`; deviation 0
- Confirmed accumulation via human adopt (not threshold alone): confirmed=4 at end
- Best / Semi-best contraction logged after formal phase

## Design notes

- HashA = coordinate frame (absolute reference)
- HashB = trajectory (relative state)
- Confirmed ≔ human-adopted facts inside the AbsoluteAnchor

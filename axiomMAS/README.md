# axiomMAS

Impurity-Aware Multi-Agent System (Go prototypes)

## Files

### v3.2.2 (current)
- `axiom_mas_v3.2.2_AbsoluteAnchor.go` — Dual Hash + **AbsoluteAnchor** + Human Gate  
  (full source: see experiment report + conversation artifact; if only `.b64.part*` are present, run `bash restore_go.sh`)
- `MAS_Experiment_Report_v3.2.2.md` — Experiment report (knowledge-bot answer policy)
- `restore_go.sh` — restores Go source from base64+gzip parts if used

### v2.7 (legacy)
- `axiom_mas_v27.go`
- `axiom-mas-v27-experiment-zenn.md`

## Quick run (v3.2.2)

```bash
# If using compressed parts:
bash restore_go.sh
go run axiom_mas_v3.2.2_AbsoluteAnchor.go
```

## v3.2.2 highlights

- AbsoluteAnchor continuity (same HashA across 12 capsules)
- Invariant: `lawful:simulation_must_not_violate_law`
- R7 alignment includes `law` axis
- Human Gate: possible / impossible / ask_human → adopt → Confirmed
- Experiment end state: confirmed=4

## Design notes

- HashA = coordinate frame
- HashB = trajectory
- Confirmed ≔ human-adopted facts inside AbsoluteAnchor

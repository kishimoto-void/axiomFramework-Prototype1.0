# Python Capsule Memory Demo (DualHash + Ζ)

Prototype: multi-layer capsule state for LLM agent governance.

**Status:** experiment card **8/8 PASS** (identity fixed / pressure moves / human-gated promotion).

## Files

| Path | Role |
|------|------|
| `axiom_memory_capsule_demo.py` | Runnable demo entry (full single-file implementation) |
| `README.md` | This note |

## Layers

| Layer | Role | Hash |
|-------|------|------|
| α0 | Genesis constraints | Hash-A |
| θ | Attachable extra policy | Hash-θ (**not** in Hash-A) |
| β | Identity core | Hash-A |
| δ | Relational | Hash-B |
| γ | Episodic memory | Hash-B |
| ε (Δ) | Working / short-term | Hash-B |
| Ζ | Observed pressure field | **not stored** |

## Run

```bash
cd examples/python
python axiom_memory_capsule_demo.py
```

Expect **8/8 PASS** on the experiment card (A–H).

## Design notes

- Hash-A = α0 + β only (immutable identity frame)
- Hash-B = outer trajectory; import checks claimed vs computed + trusted provenance
- Jailbreak := α0 violation; θ violation := policy_violation
- Ζ threshold sticky attachment → ε→γ promotion **request**; write only after human approve
- Confirmed = human adopt only (AXIOM)

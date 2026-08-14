# AXIOM Framework — Prototype 1.0 (Home)

**Milestone**: Difference Convergence Observation  
**Status**: **1.0.0** — CTS Full PASS  
**Date**: 2026-08-14  
**License**: **TBD** (see below)

> 実験は忠実に実際行って

本リポジトリは **AXIOM の本家（home）** として運用します。

---

## License Status / ライセンスについて

### English

This project is currently released as a **research prototype**.

The **final licensing model has not yet been determined**. It will be announced when the framework reaches a stable release (planned around product v1.x).

Reasons the license is left open at this stage:

- Impact of a chosen license on future adoption and generality is still under evaluation
- Growth path (research-only vs dual research/commercial, multi-language reference, etc.) is not fixed
- Premature lock-in would make later switches to MIT, Apache-2.0, or a custom license harder

**Until a stable release announces a final license, all rights are reserved by the author / copyright holders**, except where a file or crate explicitly states otherwise.

Use for personal evaluation, academic study, and non-production experiments is the intended context of this prototype. For any other use, contact the author.

### 日本語

本プロジェクトは現在、**研究・プロトタイプ段階**です。

**最終的なライセンス形態は未定**です。安定版リリース（製品 v1.x 前後）の時点で決定・公開する予定です。

現時点で確定を見送っている理由:

- 選択するライセンスが、今後の普及・汎用性に与える影響をまだ見極めている段階であること
- 成長の方向性（研究専用 / 研究と商用の二系統 / 多言語リファレンス実装など）が固まっていないこと
- 早すぎる固定は、後からの MIT・Apache-2.0・独自ライセンス等への変更を難しくすること

**安定版で最終ライセンスが明示されるまで、特に明記されているものを除き、著作権は作者（著作権者）に帰属します。**

個人の評価・学術・非本番の実験を想定した公開です。それ以外の利用については作者へご連絡ください。

Draft historical texts (e.g. earlier Research License wording, per-crate notes) may remain in the tree as **non-binding drafts** until the stable-release decision.

See also: [`LICENSE`](./LICENSE)

---

## Pipeline

```
Input → PSS → PLP → Capsule → ACP → DCK → Difference Report
```

| Crate | Role |
|-------|------|
| pss / pss-spec | Normalize + Problem Spec |
| plp | State projection (PLP-R) |
| capsule | A/B dual-hash storage |
| acp | Seal / Frame / coordinate v1.2.0 |
| dck | Difference taxonomy v2.3 |
| runtime | Wire-up |

---

## Conformance (CTS v1.0.0)

```bash
cargo test -p axiom-plp --test cts_v1
```

- Spec: [`CONFORMANCE.md`](./CONFORMANCE.md) · [`VERSIONING.md`](./VERSIONING.md)
- Levels: **Core** / **Full** / Experimental
- Latest report: **Full — 14/14 PASS**

CI pass requires: all Full tests · Golden Lock · Determinism ×100 · baseline divergence 0.

---

## Principles (POLICY)

| Principle | Meaning |
|-----------|---------|
| History is Truth | Sealed states are not rewritten |
| Canonical is Immutable | New state ⇒ new Capsule / Seal |
| Difference is Observable | DCK always emits measurable metrics |
| Projection is Replaceable | Projectors are swappable |
| Framework before Model | No vendor LLM required for the core loop |

---

## Related

- Legacy research mirror: [Axiom-Framework](https://github.com/kishimoto-void/Axiom-Framework)

*Difference Convergence Observation — Prototype 1.0 is home.*

# AXIOM Prototype Golden Vector v1.0

**Status**: Locked for Prototype 1.0 observation points  
**Date**: 2026-08-15  
**Scope**: `PSS → PLP → Capsule → ACP → DCK` pipeline invariants  
**Philosophy**: 13個の観測点として、AXIOMの不変条件を機械的に固定する

> 実験は忠実に実際行って

本ディレクトリの13本は「テスト数を増やした」ものではない。  
AXIOM Prototype の **不変性・決定性・差異検出** を、将来の変更でも壊れない基準点として固定するための **Golden Vectors** である。

特に重要なのは:

- **10 / 11** — Hash A（不変契約）と Hash B（可変観測）の分離を機械検証
- **13** — DCK の「差異を観測して単調に収束する」基本挙動を固定

これらが通れば、READMEに書かれた設計思想が **コード上で検証可能な契約** になる。

---

## 設計意図（なぜ13本か）

現在の Prototype 構成を踏まえ、以下の層を意識して選んだ。

| 層 | 役割 | 対応 Vector |
|----|------|-------------|
| PSS / Input | 正規化・入力安定性 | 01–08 |
| Capsule | 決定論的コンテナ再生成 | 09 |
| ACP | Hash A 封印 / 不変規則 | 10–11 |
| DCK | 差異観測・収束判定 | 12–13 |

CTS v1.0（`cts_v1`）とは別に運用する。  
CTS は「実装が仕様に適合するか」を、Golden Vector は「AXIOMの核心不変条件が壊れていないか」を観測する。

---

## 13 Golden Vectors

| # | ID | テスト目的 | 検証内容 |
|---|-----|------------|----------|
| 01 | `baseline_identity` | 基準入力 | 同一入力 → 同一 Hash（決定性） |
| 02 | `input_single_delta` | 入力1箇所変更 | Hash / Difference が変化する |
| 03 | `input_order_stability` | 入力順序差 | Canonical化後は同一 |
| 04 | `numeric_boundary_min` | 数値下限 | 境界値処理が決定論的 |
| 05 | `numeric_boundary_max` | 数値上限 | 境界値処理が決定論的 |
| 06 | `numeric_precision` | 小数精度 | 正規化後 Hash 決定性 |
| 07 | `unicode_normalization` | Unicode | Canonical 表現の同一性 |
| 08 | `empty_optional_fields` | Optional空値 | 明示空 / 省略の扱いが一致 |
| 09 | `capsule_determinism` | Capsule再生成 | 同一 Projection → 同一 Capsule Hash |
| 10 | `acp_hash_a_invariance` | 不変規則 | 観測変更でも Hash A が変化しない |
| 11 | `acp_hash_b_variation` | 可変観測状態 | Hash B のみ変化する |
| 12 | `dck_zero_difference` | 完全一致 | `diff = 0` / converged |
| 13 | `dck_monotonic_convergence` | 段階的収束 | difference が単調減少して 0 になる |

### 特に13番の意義

```
baseline
   ↓
difference = large
   ↓
difference = medium
   ↓
difference = small
   ↓
difference = 0
```

これを固定しておけば、「差異を観測して収束を判定する」という DCK の基本挙動が、将来の変更で壊れていないかを CI で毎回確認できる。

### 10・11 の意義（Hash A / Hash B 分離）

```
同じ規則 + 観測だけ変更
        ↓
Hash A == Hash A'
Hash B != Hash B'
```

これが通れば、Prototype の設計思想である

> 「不変な契約」と「変化する観測」を同一 Hash に混ぜない

が、単なる README の思想ではなく **機械的に検証できる契約** になる。

---

## ファイル構成

```
tests/
└── golden_vectors/
    ├── AXIOM_PROTOTYPE_GOLDEN_VECTORS_v1.0.md   ← 本ドキュメント
    ├── baseline_identity.json
    ├── input_single_delta.json
    ├── input_order_stability.json
    ├── numeric_boundary_min.json
    ├── numeric_boundary_max.json
    ├── numeric_precision.json
    ├── unicode_normalization.json
    ├── empty_optional_fields.json
    ├── capsule_determinism.json
    ├── acp_hash_a_invariance.json
    ├── acp_hash_b_variation.json
    ├── dck_zero_difference.json
    └── dck_monotonic_convergence.json
```

各 JSON は **言語非依存** に読める形（Python / Rust / Go などから同じ Vector を参照可能）にしている。

---

## 運用方針

1. **Once LOCKED, do not rewrite in place**  
   ハッシュ値や期待値が確定したら、そのファイルは書き換えない。新しい観測点は新しい ID / 新バージョンで追加する。

2. **CTS との関係**  
   - CTS v1.0 = 実装適合性ゲート  
   - Golden Vector v1.0 = 不変条件の観測点固定  
   両者は補完関係。CI では両方を走らせることを推奨。

3. **拡張**  
   v1.1 以降で新しい観測点を追加する場合は、本ドキュメントに追記し、新しい JSON を追加する。既存13本の意味を変えない。

---

## 次のアクション（推奨）

- [ ] 各 Vector に対する最小 runner（Rust `#[test]` または Python）を `crates/plp/tests/` または `tests/` に追加
- [ ] CI で `golden_vectors/*.json` の存在と schema をチェック
- [ ] 実測で得られた canonical / dual-hash を各 JSON の `expected` にロック（v1.0.1）
- [ ] Cross-language（Rust / Python）で同じ Vector を読み、同一結果を確認

---

*AXIOM Prototype Golden Vector v1.0 — 不変条件を13個の観測点として固定する*  
*実験は忠実に実際行って*

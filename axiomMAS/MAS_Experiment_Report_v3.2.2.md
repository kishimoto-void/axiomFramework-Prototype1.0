# AXIOM-MAS v3.2.2 実験レポート

**実験日時**: 2026-08-21  
**対象コード**: `axiom_mas_v3.2.2_AbsoluteAnchor.go`  
**プロトコル**: 12ラウンド（仮組み R0–5 / 本組 R6–11）  
**座標系**: AbsoluteAnchor（HashA）固定 + HashB 軌跡

---

## 1. 実験目的

1. 仮組み（逆算）と本組（順算＋検証）で何が修正・保持されるかを観察する
2. R7（本組開始）で AbsoluteAnchor からどれだけ逸脱しないかを確認する
3. 最終的にベスト / 準ベストがどう収縮するかを記録する
4. 「シミュレーションであっても法を犯さない」が HashA 不変制約として機能するかを確認する

制御層（Capsule / DualHash / AbsoluteAnchor / Monitor）の成熟度測定が主目的。  
推論の賢さ自体は二の次。

---

## 2. 問題設定

| 項目 | 内容 |
|------|------|
| 問題 | 社内ナレッジ検索ボットの回答方針を決める |
| 最終目標 C | 社内ナレッジに基づき、正確で有用な回答を返す。個人情報・機密・法的に禁止された開示は行わない |
| C ID | `goal-C:accurate+useful+lawful-disclosure` |
| 骨格 | `retrieve → ground → answer → refuse-if-unlawful` |
| 追加制約 | `no-pii-leak`, `no-confidential-leak`, `non-contradiction`, `resource-bound` |
| **不変制約（HashA）** | **`lawful:simulation_must_not_violate_law`**（シミュレーションであっても法を犯さない） |

### AbsoluteAnchor 構成

| フィールド | 値 |
|------------|-----|
| StabilityClass | `strict` |
| DimensionalAxes | `problem`, `goal`, `constraint`, `skeleton`, `law` |
| MaxDeviation | 0.34 |
| StrictOnAxes | `goal`, `constraint`, `law` |
| 固定 HashA（先頭16桁） | `198000ea2a7a4bf6` |

---

## 3. プロトコル概要

```
Turn 1 仮組み（逆算）          Turn 2 本組（順算＋検証）
R0 問題確認                    R6  R7照合（AbsoluteAnchor）
R1 骨格定義                    R7  順算で骨格再構築
R2 C達成シナリオ逆算            R8  ABCシナリオ想定
R3 B条件                       R9  整合性チェック
R4 A条件                       R10 実現可能性
R5 CBA仮組みの閉じ              R11 ベスト / 準ベスト出力
```

- HashA = 座標系（原則不動）
- HashB = 軌跡（ラウンドごとに変化）
- 全 Capsule に同じ AbsoluteAnchor を添付

---

## 4. 実行結果サマリ

| 指標 | 結果 |
|------|------|
| Self-tests | ✅ 通過（DualHash / Jaccard / Wire / AbsoluteAnchor） |
| Capsules sealed | **12** |
| AbsoluteAnchor continuity | **true**（全 Capsule が同一 HashA） |
| R7 aligned | **true** |
| R7 deviation | **0.0** |
| R7 matched axes | **5**（`constraint`, `goal`, `law`, `problem`, `skeleton`） |
| R7 drifted axes | **0** |
| ConfirmedState（終了時） | **4**（Human Gate: adopt のみ昇格） |
| TentativeState（終了時） | 4 |
| Blacklist | 0 |
| Monitor | 主に `Caution`（Compound） |
| Human Gate | possible / impossible / ask_human → adopt/reject |

---

## 5. ラウンド別メモ

### Turn 1：仮組み（R0–5）

- 全ラウンドで `anchor_attached=true`
- Monitor は R0 のみ `Continue`、以降は `Caution`（DualHash Class = Compound）
- 法・goal・constraint を意識した Reasoning / Content は Mock から出力済み

### R6（R7 照合）

```
aligned=true
deviation=0
matched_axes=[constraint goal law problem skeleton]
drifted_axes=[]
stability=strict
```

- 仮組み成果が固定座標系の内側に収まっている
- **`law` 軸もマッチ**（不変制約が照合に効いている）
- 逸脱なし → 本組を Caution 付きで継続

### Turn 2：本組（R7–11）

- 引き続き同一 HashA を継承
- HashB のみがラウンドごとに変化（軌跡）
- 最終ラウンドでベスト / 準ベストを確定出力
- Human Gate で Confirmed を蓄積（R1/R3/R7 で adopt）

---

## 6. 最終出力：ベスト / 準ベスト

### 🏆 ベスト

| 項目 | 内容 |
|------|------|
| 方針 | 根拠文書があるときだけ回答。PII・機密・法的禁止は必ず拒否。拒否理由を簡潔に返す |
| 法 | シミュレーションであっても法を犯さない（HashA 不変制約） |
| 骨格 | `retrieve → ground → answer → refuse-if-unlawful` |
| トレードオフ | **有用性より合法性・正確性を優先** |

### 🥈 準ベスト

| 項目 | 内容 |
|------|------|
| 方針 | 該当文書の存在メタまでは返せるが、本文・個人情報・機密は出さない |
| 法 | 本文開示はせず、法・契約境界は維持 |
| トレードオフ | **有用性は上がるが、情報漏洩リスクの解釈幅が残る** |

---

## 7. 設計仮説との対応

| 仮説 | 実験での様子 |
|------|----------------|
| HashA は座標系、HashB は軌跡 | 全 12 Capsule で HashA 固定、HashB のみ変化 ✅ |
| 法はソフトポリシーではなく HashA 不変項 | ConstraintSet 先頭に固定、R7 で `law` マッチ ✅ |
| 仮組み → 本組の二重化 | R0–5 仮 / R6 照合 / R7–11 本 として分離 ✅ |
| R7 で逸脱検出可能 | deviation=0 で「内側に収まっている」ことを確認 ✅ |
| ベスト＋準ベストで探索余地を残す | 両方ログ出力 ✅ |
| Confirmed が実質的に蓄積する | Human adopt により **confirmed=4** ✅ |

---

## 8. 考察

### うまくいった点

1. **AbsoluteAnchor の連続性**  
   生成時刻を Hash 材料から外したため、同一条件なら同一 HashA。12 Capsule すべてで一致を確認できた。

2. **法制約の位置づけ**  
   「シミュレーションでも法を犯さない」を座標系に入れたことで、R7 照合の軸として自然に効いた。

3. **Human Gate**  
   Confirmed ≔ 人間が adopt した項目。impossible は adopt 不可。

### まだ弱い点

1. 対話的 Human Gate（stdin/API）はデモではシミュレーション入力
2. R7 照合がまだ記号的（軸名部分一致）
3. ベスト／準ベストの状態からの自動導出は今後

---

## 9. 結論

- HashA 連続性：成功  
- 法制約の HashA 埋め込みと R7 での `law` マッチ：成功  
- ベスト／準ベストの二段出力：成功  
- Confirmed 蓄積：Human Gate（adopt）により解消方向

## 付録：再現方法

```bash
go run axiom_mas_v3.2.2_AbsoluteAnchor.go
```

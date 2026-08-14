# GitHub Actions — CI 手順

**目的**: 毎コミットで「仕様（CTS Full）を満たしている」ことを自動確認する。

```
Push / PR
  ↓
cargo fmt --check
  ↓
cargo clippy -- -D warnings
  ↓
cargo test --workspace
  ↓
CTS v1.0 (14 tests) + Golden Lock
  ↓
PASS ✅
```

---

## 1. ファイル配置

| Path | 役割 |
|------|------|
| `.github/workflows/ci.yml` | 本パイプライン |
| `crates/plp/tests/cts_v1.rs` | Conformance Suite |
| `tests/golden_vectors/PLP_R_GOLDEN_LOCK_v0_1.json` | Golden Lock |
| `CONFORMANCE.md` | 合格条件の仕様 |

---

## 2. トリガー

- `push` → `main`
- `pull_request` → `main`
- `workflow_dispatch`（Actions タブから手動）

---

## 3. Jobs

| Job | コマンド | 失敗時 |
|-----|----------|--------|
| **fmt** | `cargo fmt --all -- --check` | フォーマット不一致 |
| **clippy** | `cargo clippy --workspace --all-targets -- -D warnings` | lint / 警告 |
| **test** | `cargo test --workspace --all-targets` | ユニット・結合失敗 |
| **cts** | `cargo test -p axiom-plp --test cts_v1` + Golden メタ検証 | CTS / Golden 不一致 |
| **ci-success** | needs 全成功 | ゲート集約 |

Toolchain: **stable**（`dtolnay/rust-toolchain`）+ rustfmt / clippy  
Cache: `Swatinem/rust-cache@v2`

---

## 4. ローカルで同じことをする

```bash
# 1. format
cargo fmt --all -- --check

# 2. clippy
cargo clippy --workspace --all-targets -- -D warnings

# 3. tests
cargo test --workspace --all-targets

# 4. CTS (Full gate)
cargo test -p axiom-plp --test cts_v1 -- --nocapture
```

一発スクリプト例:

```bash
#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p axiom-plp --test cts_v1
echo "CI local PASS"
```

---

## 5. GitHub 上の有効化手順

1. 本 workflow が `main` にマージされていること（このドキュメントと同時に追加済み）
2. リポジトリ → **Actions** タブで workflow が表示されることを確認
3. （推奨）**Settings → Branches → Branch protection** で `main` に対し Required status checks に次を指定:
   - `cargo fmt`
   - `cargo clippy`
   - `cargo test`
   - `CTS v1.0 (Full)`
   - または集約ジョブ `CI success`
4. 以降、PR は上記が緑になるまでマージ不可にできる

---

## 6. 合格条件（CONFORMANCE と一致）

- CTS Full（機能 13 + メタデータ）PASS
- Golden Lock: `schema_version=1.0`, `cts_version=1.0.0`, vectors=4
- Determinism / baseline は CTS 内で検証

詳細: [`CONFORMANCE.md`](../CONFORMANCE.md)

---

## 7. 将来の拡張（未実装・ロードマップ）

| 拡張 | 内容 |
|------|------|
| Matrix toolchain | `1.75` / `stable` / `beta` |
| OS matrix | `ubuntu-latest` / `windows-latest` / `macos-latest` |
| Release | tag `v*` で artifacts / GitHub Release |
| ACP Full | `cargo test -p axiom-acp` を別 job（toolchain 要件付き） |

---

*品質保証（QA）段階 — 回帰を早く検出する*

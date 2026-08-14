# PLP State Projection — Research Note (PLP-R)

**Research revision**: v0.1.2  
**Payload version** (serialized, Golden-locked): `0.1.1`  
**Status**: Research Prototype — Phase 1–3 landed in Prototype 1.0  
**Production**: PLP Capsule v1.1.3（並列・非破壊、Axiom-Framework）

---

## 設計契約（不変）

1. **PLP は意味解析をしない**  
2. **Annotation = Canonical Projection Candidate**（Semantic Truth ではない）  
3. **3層**: Raw Text / Canonical State / Dual Hash  
4. **Dual Hash**

```
raw_hash        = SHA256(raw_text)           → HashA
canonical_hash  = SHA256(header + Canonical) → HashB
```

Raw を canonical_hash に入れない（状態同一性を表現ゆれから独立）。

---

## Projector 階層

| Projector | 出力 |
|-----------|------|
| TokenOnly | language + tokens（annotations 空, status=none） |
| Minimal | + candidate annotations（status=canonical_projection_candidate） |

---

## Phase 進捗（Prototype 実装）

| Phase | 内容 | 状態 |
|-------|------|------|
| 1 | Golden Vector | lock ファイル移行済 |
| 2 | DCK 接続 | bridge 契約 + `axiom-dck` v2.3 |
| 3 | Monitor | `monitor.rs` Continue/AskUser/Abort |

Rust: `crates/plp/` · Docs: `docs/plp-r/`

---

*実験は忠実に実際行って*

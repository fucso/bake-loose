# Task: Cargo.toml 依存クレート追加

> Feature: [get-project](../../spec.md)
> 依存: なし

## 目的
プロジェクトに必要な依存クレートを追加する。GraphQL、DB接続、UUID生成などの基盤となるクレートを揃える。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/Cargo.toml` | 修正 | 依存クレートの追加 |
| `backend/Cargo.lock` | 自動生成 | cargo build で生成、コミット対象 |

---

## 設計詳細

### 追加する依存クレート

| クレート | バージョン | 用途 |
|----------|-----------|------|
| `uuid` | `1` | UUID 生成・パース（features: `v4`, `serde`） |
| `async-trait` | `0.1` | 非同期トレイトのサポート |
| `thiserror` | `2` | エラー型の derive マクロ |
| `chrono` | `0.4` | 日時操作（features: `serde`） |
| `dotenvy` | `0.15` | .env ファイルの読み込み |
| `tracing` | `0.1` | ロギング |
| `tracing-subscriber` | `0.3` | ロギングサブスクライバー |

### 既存クレートの features 追加

| クレート | 追加する features |
|----------|------------------|
| `sqlx` | `uuid`, `chrono` |

---

## 完了条件

- [ ] 上記クレートが Cargo.toml に追加されている
- [ ] `cargo build` が成功する
- [ ] Cargo.lock がコミットに含まれている

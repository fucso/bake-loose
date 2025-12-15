# Task: Infrastructure層（DB接続プール）

> Feature: [get-project](../../spec.md)
> 依存: 01-dependencies

## 目的
PostgreSQL への接続プールを作成する機能を実装する。環境変数から接続URLを受け取り、接続プールを返す純粋な技術機能を提供する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/infrastructure.rs` | 新規 | DB接続プール作成関数 |
| `backend/src/lib.rs` | 新規 | クレートルートでのモジュール公開 |

---

## 設計詳細

### infrastructure.rs

DB接続プールの作成関数を実装する。接続URLは引数として受け取る（環境変数からの読み込みは呼び出し側の責務）。

- `create_pool(url: &str) -> Result<PgPool, sqlx::Error>`
  - SQLx の `PgPoolOptions` を使用
  - 最大接続数: 5（開発環境向けの小さな値）

### Infrastructure層のポイント

- ドメイン層への依存なし
- 環境変数の読み込みは main.rs で行う（1ファイル1関心事の原則）
- SQL の発行は行わない（それは repository層の責務）

---

## 完了条件

- [ ] `backend/src/infrastructure.rs` が作成されている
- [ ] `create_pool` 関数が実装されている
- [ ] lib.rs で infrastructure モジュールが公開されている
- [ ] `cargo check` が成功する

# Task Report: テスト基盤のセットアップ

> 実施日時: 2025-12-31
> 依存タスク: なし

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/Cargo.toml` | 修正 | dev-dependencies 更新（sqlx に `macros` feature 追加、不要クレート削除） |
| `backend/tests/graphql/schema.rs` | 新規 | テスト用スキーマビルダー `create_test_schema()` |
| `backend/tests/graphql.rs` | 修正 | `schema` モジュール追加 |
| `backend/tests/fixtures/projects.sql` | 新規 | Project テストデータ（2件） |

## ビルド・テスト結果

### cargo check

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 25.66s
```

警告なし。

### cargo test

このタスクではテスト基盤のセットアップのみ。実際のテスト移行は次タスク（02-migrate-existing-tests）で実施。

## 設計上の議論と決定

### sqlx の feature 名

**議論**:
- タスク仕様 (spec.md) では `test-util` feature と記載されていたが、実際には存在しない feature だった

**検討した選択肢**:

1. **`macros` feature（採用）**
   - `#[sqlx::test]` マクロを有効化する正しい feature
   - [公式ドキュメント](https://docs.rs/sqlx/latest/sqlx/attr.test.html)で確認

2. **`test-util` feature（不採用）**
   - 存在しない feature（コンパイルエラー）

**決定理由**:
- 公式ドキュメントに基づき、正しい feature 名 `macros` を採用

## 先送り事項

- [ ] `create_test_schema` 関数が未使用（02-migrate-existing-tests で使用予定）
- [ ] `fixtures/projects.sql` が未使用（02-migrate-existing-tests で使用予定）

## 次タスクへの申し送り

1. **テスト用スキーマビルダーの使い方**:
   ```rust
   use crate::graphql::schema::create_test_schema;

   #[sqlx::test(migrations = "./migrations", fixtures("projects"))]
   async fn test_example(pool: PgPool) {
       let schema = create_test_schema(pool);
       let response = schema.execute("{ ... }").await;
   }
   ```

2. **フィクスチャの配置**:
   - `tests/fixtures/` 配下に SQL ファイルを配置
   - `#[sqlx::test(fixtures("ファイル名"))]` で参照（拡張子不要）

3. **spec.md の修正推奨**:
   - `test-util` → `macros` に修正が必要

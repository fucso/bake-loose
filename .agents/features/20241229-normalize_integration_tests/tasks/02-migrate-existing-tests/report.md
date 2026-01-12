# Task Report: 既存テストの移行

> 実施日時: 2025-12-31
> 依存タスク: 01-setup-sqlx-test

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/tests/graphql/schema.rs` | 修正 | `execute_graphql` ヘルパー関数を追加 |
| `backend/tests/graphql/projects/project.rs` | 修正 | `sqlx::test` + 完全 JSON 検証に移行 |
| `backend/tests/graphql/projects/projects.rs` | 修正 | `sqlx::test` + 完全 JSON 検証に移行 |
| `backend/tests/graphql/request.rs` | 削除 | HTTP ベースのヘルパー |
| `backend/tests/graphql/test_data.rs` | 削除 | モジュール定義 |
| `backend/tests/graphql/test_data/project.rs` | 削除 | 手動データ操作ヘルパー |
| `backend/tests/graphql.rs` | 修正 | `request`, `test_data` モジュールを削除 |
| `.claude/rules/testing.md` | 修正 | GraphQL テストルールを追記 |

## ビルド・テスト結果

### cargo check

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
```

警告なし。

### cargo test

```
running 10 tests (lib)     ... ok. 10 passed
running 4 tests (graphql)  ... ok. 4 passed
```

全 14 テスト成功。並列実行も正常動作。

## 設計上の議論と決定

### フィクスチャのパス指定

**議論**:
- 当初 `fixtures("projects")` と指定したが、`sqlx::test` はテストファイルのディレクトリを基準にフィクスチャを探すためエラーになった

**検討した選択肢**:

1. **相対パス指定（採用）**
   - `fixtures("../../fixtures/projects.sql")` で明示的にパスを指定
   - フィクスチャを `tests/fixtures/` に集約できる

2. **各テストディレクトリにフィクスチャ配置（不採用）**
   - `tests/graphql/projects/fixtures/projects.sql` に配置
   - フィクスチャが分散し、再利用が困難

**決定理由**:
- フィクスチャの一元管理と再利用性を優先

### クエリ実行ヘルパーの追加

**議論**:
- レビューで「スキーマ構築からレスポンス取得までを共通化したい」という要望があった

**検討した選択肢**:

1. **`execute_graphql` 関数（採用）**
   - `pool` と `query` を受け取り、JSON を返す
   - エラーチェックも内包

2. **`create_test_schema` のみ（不採用）**
   - 各テストでスキーマ構築、クエリ実行、エラーチェックを記述
   - ボイラープレートが多い

**決定理由**:
- テストコードの簡潔さと一貫性を優先

### GET 系テストの検証方法

**議論**:
- 部分一致 vs 完全一致の検証方法

**検討した選択肢**:

1. **完全 JSON 検証（採用）**
   - `assert_eq!(data, json!({ ... }))`
   - 予期しないフィールドの追加を検出可能

2. **部分一致検証（不採用）**
   - `assert!(data["project"]["id"] == "...")`
   - 一部のフィールドのみ検証

**決定理由**:
- API のレスポンス形式を厳密に保証するため

## 先送り事項

なし

## 次タスクへの申し送り

この Feature の全タスクが完了しました。以下が新しいテスト基盤の使い方です:

1. **テストの書き方**:
   ```rust
   use serde_json::json;
   use sqlx::PgPool;
   use crate::graphql::schema::execute_graphql;

   #[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/xxx.sql"))]
   async fn test_xxx(pool: PgPool) {
       let data = execute_graphql(pool, "{ ... }").await;
       assert_eq!(data, json!({ ... }));
   }
   ```

2. **フィクスチャ追加時**:
   - `tests/fixtures/` に SQL ファイルを配置
   - `fixtures("../../fixtures/ファイル名.sql")` で参照

3. **コーディングルール**:
   - `.claude/rules/testing.md` に GraphQL テストのルールを追記済み

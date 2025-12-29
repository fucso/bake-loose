# Task Report: main.rs の統合・Axum ルーティング

> 実施日時: 2025-12-29
> 依存タスク: 08-presentation

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/tests/graphql.rs` | 新規 | GraphQL統合テストのエントリーポイント |
| `backend/tests/graphql/request.rs` | 新規 | テスト用リクエストユーティリティ |
| `backend/tests/graphql/test_data.rs` | 新規 | テストデータモジュール宣言 |
| `backend/tests/graphql/test_data/project.rs` | 新規 | プロジェクトテストデータ操作 |
| `backend/tests/graphql/projects.rs` | 新規 | プロジェクト関連テストモジュール宣言 |
| `backend/tests/graphql/projects/project.rs` | 新規 | 単一プロジェクト取得テスト |
| `backend/tests/graphql/projects/projects.rs` | 新規 | プロジェクト一覧取得テスト |
| `backend/src/constant/env.rs` | 修正 | TEST_DATABASE_URL サポート追加 |
| `backend/Cargo.toml` | 修正 | serial_test 依存追加 |

## ビルド・テスト結果

### cargo check

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
```

### cargo test

```
running 10 tests
test domain::models::project::tests::test_project_id_new_generates_unique_ids ... ok
test domain::models::project::tests::test_project_new_creates_with_auto_id ... ok
test use_case::project::get_project::tests::test_get_project_returns_specified_project_from_multiple ... ok
test use_case::project::list_projects::tests::test_list_projects_empty ... ok
test use_case::project::get_project::tests::test_get_project_not_found ... ok
test use_case::project::list_projects::tests::test_list_projects_returns_sorted_by_name_asc ... ok
test repository::project_repo::tests::test_find_by_id_returns_none_when_not_exists ... ok
test repository::project_repo::tests::test_find_by_id_returns_project_when_exists ... ok
test repository::project_repo::tests::test_find_all_with_name_asc ... ok
test repository::project_repo::tests::test_find_all_with_created_at_desc ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

running 4 tests
test graphql::projects::project::test_returns_null_when_not_found ... ok
test graphql::projects::project::test_returns_project ... ok
test graphql::projects::projects::test_contains_inserted_project ... ok
test graphql::projects::projects::test_returns_list ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
```

## 設計上の議論と決定

### テスト用DBコネクションプールの管理方式

**議論**:
- テストが並列実行されるとコネクションプールの競合が発生し、タイムアウトエラーが発生
- `static OnceCell<PgPool>` でプールを共有する方式では Tokio ランタイムのシャットダウン時にエラーが発生

**検討した選択肢**:

1. **static OnceCell でプールを共有（不採用）**
   - テスト間でプールを再利用できる
   - Tokio ランタイムのシャットダウン順序に起因する不安定さがあった

2. **max_connections を増やす（不採用）**
   - テストの都合でアプリケーション設定を変更するのは不適切

3. **各テストで新規プールを作成（採用）**
   - テストごとに独立したプールを作成
   - テスト終了時にプールが自動的にドロップされる
   - シンプルで安定した動作

**決定理由**:
- テストの独立性が保たれる
- Tokio ランタイムとの相性問題を回避
- 実行時間への影響は軽微（各テスト約0.4秒）

### テストの並列実行制御

**議論**:
- DBを使用するテストが並列実行されるとデータ競合が発生する可能性

**決定**:
- `serial_test` クレートの `#[serial]` アトリビュートを使用
- GraphQL統合テストは順次実行される

## 先送り事項

なし

## 次タスクへの申し送り

- 統合テストのパターンは `tests/graphql/` 以下を参照
- 新しいエンティティのテストを追加する場合は `test_data/` にヘルパー関数を追加
- `TEST_DATABASE_URL` を設定すると本番DBとテストDBを分離可能

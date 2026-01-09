# Task Report: Repository層（PostgreSQL実装）

> 実施日時: 2025-01-06
> 依存タスク: 02-infrastructure, 03-migration, 05-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/repository.rs` | 新規 | repository モジュール宣言 |
| `backend/src/repository/models.rs` | 新規 | DBモデルサブモジュール |
| `backend/src/repository/models/project_row.rs` | 新規 | ProjectRow と From トレイト実装 |
| `backend/src/repository/project_repo.rs` | 新規 | PgProjectRepository 実装 + テスト |
| `backend/src/main.rs` | 修正 | `mod repository;` 追加 |
| `compose.yaml` | 修正 | TEST_DATABASE_URL 追加、init スクリプトマウント |
| `docker/postgres/init-test-db.sql` | 新規 | テスト用DB作成スクリプト |
| `README.md` | 修正 | テスト用DBセットアップ手順追加 |

## ビルド・テスト結果

### cargo check

```
warning: unused import: `error::RepositoryError` (src/ports.rs:9:9)
warning: unused import: `project_repository::ProjectRepository` (src/ports.rs:10:9)
warning: unused import: `project_repo::PgProjectRepository` (src/repository.rs:8:9)
warning: struct `ProjectId` is never constructed (src/domain/models/project.rs:8:12)
warning: struct `Project` is never constructed (src/domain/models/project.rs:25:12)
warning: function `create_pool` is never used (src/infrastructure/database.rs:10:14)
warning: enum `RepositoryError` is never used (src/ports/error.rs:5:10)
warning: trait `ProjectRepository` is never used (src/ports/project_repository.rs:8:11)
warning: struct `ProjectRow` is never constructed (src/repository/models/project_row.rs:11:12)
warning: struct `PgProjectRepository` is never constructed (src/repository/project_repo.rs:13:12)

Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
```

### cargo test

```
running 5 tests
test domain::models::project::tests::test_project_id_new_generates_unique_ids ... ok
test domain::models::project::tests::test_project_new_creates_with_auto_id ... ok
test repository::project_repo::tests::test_find_by_id_returns_none_when_not_exists ... ok
test repository::project_repo::tests::test_find_by_id_returns_project_when_exists ... ok
test repository::project_repo::tests::test_find_all_returns_projects_ordered_by_created_at_desc ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 先送り事項

- [ ] 未使用警告（use_case/presentation 層で使用予定のため現時点では想定通り）
  - `PgProjectRepository`, `ProjectRow` 未使用
  - `create_pool` 未使用
  - `RepositoryError`, `ProjectRepository` 未使用

## 次タスクへの申し送り

- `PgProjectRepository::new(pool)` でリポジトリを生成できる
- テストは `TEST_DATABASE_URL` 環境変数を使用（開発用DBとは分離）
- テスト用DBへのマイグレーションは手動で実行が必要: `DATABASE_URL=$TEST_DATABASE_URL sqlx migrate run`

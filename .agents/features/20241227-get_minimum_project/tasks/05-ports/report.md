# Task Report: Ports層（ProjectRepository トレイト）

> 実施日時: 2025-12-28
> 依存タスク: 04-domain

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/ports/error.rs` | 新規 | RepositoryError 定義 |
| `backend/src/ports/project_repository.rs` | 新規 | ProjectRepository トレイト定義 |
| `backend/src/ports.rs` | 新規 | ports モジュール（サブモジュールを再公開） |
| `backend/src/main.rs` | 修正 | `mod ports;` 宣言を追加 |

## ビルド・テスト結果

### cargo check

```
warning: unused import: `error::RepositoryError`
 --> src/ports.rs:9:9

warning: unused import: `project_repository::ProjectRepository`
  --> src/ports.rs:10:9

warning: struct `ProjectId` is never constructed
 --> src/domain/models/project.rs:8:12

warning: struct `Project` is never constructed
  --> src/domain/models/project.rs:25:12

warning: function `create_pool` is never used
  --> src/infrastructure/database.rs:10:14

warning: enum `RepositoryError` is never used
 --> src/ports/error.rs:5:10

warning: trait `ProjectRepository` is never used
 --> src/ports/project_repository.rs:8:11

warning: `bake-loose` (bin "bake-loose") generated 9 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### cargo test

テストなし（トレイト定義のみのため）

## 設計上の議論と決定

特になし。タスク仕様通りに実装。

## 先送り事項

- [ ] `RepositoryError`, `ProjectRepository` が未使用警告 → 06-repository, 07-use-case で使用予定
- [ ] `Project`, `ProjectId` が未使用警告 → 継続して後続タスクで使用予定
- [ ] `create_pool` が未使用警告 → 09-integration で使用予定

## 次タスクへの申し送り

### 06-repository（PostgreSQL実装）

- `use crate::ports::{ProjectRepository, RepositoryError};` で参照
- `ProjectRepository` トレイトを `PgProjectRepository` で実装
- `find_by_id`, `find_all` の2メソッドを実装

### 07-use-case（get_project, list_projects）

- `use crate::ports::{ProjectRepository, RepositoryError};` で参照
- リポジトリはトレイト経由で使用（具体実装への依存なし）

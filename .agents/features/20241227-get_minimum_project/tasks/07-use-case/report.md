# Task Report: UseCase層（get_project, list_projects）

## 完了状況

すべての完了条件を満たしました。

| 条件 | 状況 |
|------|------|
| `get_project` ユースケースが実装されている | ✅ |
| `list_projects` ユースケースが実装されている | ✅ |
| ports 層のトレイトのみに依存している | ✅ |
| `cargo check` が成功する | ✅ |

## 成果物

### 新規作成ファイル

| ファイル | 概要 |
|----------|------|
| `backend/src/use_case.rs` | use_case モジュール定義 |
| `backend/src/use_case/project.rs` | project サブモジュール |
| `backend/src/use_case/project/get_project.rs` | 単一プロジェクト取得ユースケース |
| `backend/src/use_case/project/list_projects.rs` | プロジェクト一覧取得ユースケース |
| `backend/src/use_case/test.rs` | テストユーティリティモジュール |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 共通モック（MockUnitOfWork） |
| `backend/src/ports/sort.rs` | 汎用ソート型（SortDirection, SortColumn, Sort<C>） |
| `backend/src/ports/unit_of_work.rs` | UnitOfWork トレイト |

### 修正ファイル

| ファイル | 概要 |
|----------|------|
| `backend/src/main.rs` | `mod use_case;` を追加 |
| `backend/src/ports.rs` | sort, unit_of_work モジュール追加、re-export 整理 |
| `backend/src/ports/project_repository.rs` | `ProjectSortColumn` enum、`ProjectSort` 型を追加 |
| `backend/src/repository/models/project_row.rs` | `SortColumn` trait を実装 |
| `backend/src/repository/project_repo.rs` | `find_all` でソート対応 |

## 設計上の決定

### 1. UnitOfWork パターンの導入

仕様では「将来の拡張」として記載されていましたが、以下の理由から先行して導入しました:

- 複数リポジトリを扱うユースケースでも引数が増えない
- 個別リポジトリを引数で受け取るアンチパターンを回避
- トランザクション管理の基盤を早期に確立

### 2. ソート機能の責務分離

`list_projects` の並び順（name ASC）を実現するため、以下の設計を採用:

```
ports/sort.rs              → 汎用型（モデル非依存）
ports/project_repository.rs → ソート可能フィールドの enum 定義
repository/models/project_row.rs → DB カラム名へのマッピング
```

- UseCase は「どのフィールドで並べるか」を指定
- Repository は SQL の ORDER BY 句を生成

### 3. テストユーティリティの共通化

`MockUnitOfWork` を `use_case/test/` に配置し、複数のユースケーステストで再利用可能にしました。

## テスト結果

```
running 10 tests
test domain::models::project::tests::test_project_id_new_generates_unique_ids ... ok
test domain::models::project::tests::test_project_new_creates_with_auto_id ... ok
test use_case::project::get_project::tests::test_get_project_returns_specified_project_from_multiple ... ok
test use_case::project::get_project::tests::test_get_project_not_found ... ok
test use_case::project::list_projects::tests::test_list_projects_empty ... ok
test use_case::project::list_projects::tests::test_list_projects_returns_sorted_by_name_asc ... ok
test repository::project_repo::tests::test_find_by_id_returns_none_when_not_exists ... ok
test repository::project_repo::tests::test_find_by_id_returns_project_when_exists ... ok
test repository::project_repo::tests::test_find_all_with_name_asc ... ok
test repository::project_repo::tests::test_find_all_with_created_at_desc ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## ルールの更新

以下のルールファイルを更新しました（別ブランチでコミット予定）:

- `.claude/rules/use-case.md` - UnitOfWork 経由アクセスの方針追加
- `.claude/rules/ports.md` - ソート機能の責務分離を追加
- `.claude/rules/repository.md` - SortColumn 実装の記載追加
- `.claude/rules/testing.md` - 新規作成（共通モックの配置ルール）

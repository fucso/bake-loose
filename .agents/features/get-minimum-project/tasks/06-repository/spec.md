# Task: Repository層（PostgreSQL実装）

> Feature: [get-project](../../spec.md)
> 依存: 02-infrastructure, 03-migration, 05-ports

## 目的
ports 層で定義した ProjectRepository トレイトの PostgreSQL 実装を作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/repository.rs` | 新規 | repository モジュール（サブモジュールを再公開） |
| `backend/src/repository/models.rs` | 新規 | DBモデルサブモジュール |
| `backend/src/repository/models/project_row.rs` | 新規 | ProjectRow（DBモデル） |
| `backend/src/repository/project_repo.rs` | 新規 | PgProjectRepository 実装 |
| `backend/src/lib.rs` | 修正 | repository モジュールの公開追加 |

---

## 設計詳細

### ProjectRow（DBモデル）

SQLx の `FromRow` を使用してクエリ結果をマッピングする構造体:

| フィールド | 型 | 対応カラム |
|------------|-----|-----------|
| `id` | `Uuid` | id |
| `name` | `String` | name |
| `created_at` | `DateTime<Utc>` | created_at |
| `updated_at` | `DateTime<Utc>` | updated_at |

### ドメインモデルへの変換

`From<ProjectRow> for Project` を実装:
- `Project::from_raw()` を使用して構築
- 例: `Project::from_raw(ProjectId(row.id), row.name)`

### PgProjectRepository

PostgreSQL 用のリポジトリ実装:

- `new(pool: PgPool) -> Self`: コンストラクタ
- `find_by_id`: `SELECT * FROM projects WHERE id = $1`
- `find_all`: `SELECT * FROM projects ORDER BY created_at DESC`

### Repository層の原則

- ports 層のトレイトを実装
- ビジネスロジックを含まない
- トランザクション管理は行わない（use_case層の責務）
- プレースホルダーを使用（SQLインジェクション対策）

---

## 完了条件

- [ ] `ProjectRow` が定義されている
- [ ] `From<ProjectRow> for Project` が実装されている
- [ ] `PgProjectRepository` が `ProjectRepository` を実装している
- [ ] SQL でプレースホルダーを使用している
- [ ] `cargo check` が成功する

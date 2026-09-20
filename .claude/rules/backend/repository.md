---
paths: backend/src/repository/**/*.rs
---

# Repository Layer

リポジトリ層はports層で定義されたトレイトの具体実装を担当。PostgreSQL/SQLxを使用。

## 基本原則

- **実装対象**: ports層のトレイト
- **依存先**: ports層、domain層、infrastructure層
- **禁止**: ビジネスロジック、トランザクション管理

**やること**:
- portsトレイトの実装
- SQL発行
- ドメインモデル ↔ DBモデル変換

**やらないこと**:
- ビジネスロジック
- バリデーション
- トランザクション境界の管理

## ファイル配置

```
backend/src/repository/
├── executor.rs         # PgExecutor（Pool/Transaction の抽象化）
├── pg_unit_of_work.rs  # PgUnitOfWork 実装
├── project_repo.rs
├── trial_repo.rs
├── ...
└── models/
    ├── project_row.rs
    ├── trial_row.rs
    └── ...
```

## DBモデル（Row構造体）

ドメインモデルとは別に定義し、変換処理を経由:

```rust
// src/repository/models/project_row.rs

#[derive(Debug, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    // ...
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Project::reconstruct(ProjectId(row.id), row.name, ...)
    }
}
```

## SortColumn の実装

ports層で定義された `{Model}SortColumn` に対して、DBカラム名へのマッピングを実装する:

```rust
// src/repository/models/project_row.rs

use crate::ports::project_repository::ProjectSortColumn;
use crate::ports::sort::SortColumn;

impl SortColumn for ProjectSortColumn {
    fn as_sql_column(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}
```

リポジトリでは `Sort<C>` の `to_order_by_clause()` を使用してSQLを生成:

```rust
async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError> {
    let query = format!("SELECT * FROM projects {}", sort.to_order_by_clause());
    // ...
}
```

## PgExecutor

Pool と Transaction を抽象化し、リポジトリが両方で動作できるようにする。

**設計原則**:
- sqlx の Query 型の種類ごとに Query 実行メソッドを実装する
- Pool/Transaction の分岐は `PgExecutor` 内に閉じ込め、リポジトリには露出させない
- 新しいクエリパターンが必要になった場合は `PgExecutor` にメソッドを追加する

```rust
// src/repository/executor.rs

pub enum PgExecutor {
    Pool(PgPool),
    Transaction(Arc<Mutex<Transaction<'static, Postgres>>>),
}

impl PgExecutor {
    /// 単一行を取得する（存在しない場合は None）
    pub async fn fetch_optional<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        match self {
            Self::Pool(pool) => query.fetch_optional(pool).await,
            Self::Transaction(tx) => {
                let mut guard = tx.lock().await;
                query.fetch_optional(&mut **guard).await
            }
        }
    }

    // fetch_all, fetch_one_scalar, execute, ...
}
```

## リポジトリ実装

**設計原則**:
- リポジトリは `PgExecutor` を受け取る
- SQL は一度だけ記述し、`PgExecutor` のメソッドに委譲する
- リポジトリ内で Pool/Transaction の match 分岐を書かない

```rust
// src/repository/project_repo.rs

pub struct PgProjectRepository {
    executor: PgExecutor,
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        let query = sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE id = $1")
            .bind(id.0);

        self.executor
            .fetch_optional(query)
            .await
            .map(|row| row.map(Project::from))
            .map_err(|e| map_sqlx_error(e, "project"))
    }

    // find_all, save, exists_by_name, ...
}
```

## スキーマ命名規約

DB のスキーマ名はプロジェクトのルールとして以下に統一する。PostgreSQL がデフォルトで付与する名前は
リネームせずそのまま使い、明示的に付与するのはインデックスのみとする。

| 対象 | 命名 | 備考 |
|------|------|------|
| テーブル名 | 複数形の snake_case | `projects` / `trials` / `steps` / `parameters` |
| エンティティ名（Rust） | テーブル名の単数形 | `project` / `trial` / `step` / `parameter` |
| 主キー | `{table}_pkey` | PostgreSQL のデフォルト。リネームしない |
| 一意制約 | `{table}_{columns}_key` | PostgreSQL のデフォルト。リネームしない |
| 外部キー | `{table}_{column}_fkey` | PostgreSQL のデフォルト。リネームしない。`{table}` は常に **参照する側** |
| インデックス | `idx_{table}_{columns}` | `CREATE INDEX` で明示的に付与する |

**一意インデックス**は `CREATE UNIQUE INDEX` で一意性を表現する。`uq_` / `unique_` のような
接頭辞や `_unique` / `_idx` のような接尾辞は使用しない。

### 現在の制約名・インデックス名

| 名前 | 種別 | 定義元 |
|------|------|--------|
| `projects_pkey` / `trials_pkey` / `steps_pkey` / `parameters_pkey` | 主キー | `id UUID PRIMARY KEY` |
| `steps_trial_id_position_key` | 一意制約 | `UNIQUE (trial_id, position)` |
| `trials_project_id_fkey` | 外部キー | `trials.project_id -> projects.id` |
| `steps_trial_id_fkey` | 外部キー | `steps.trial_id -> trials.id` |
| `parameters_step_id_fkey` | 外部キー | `parameters.step_id -> steps.id` |
| `idx_projects_name` | 一意インデックス | `CREATE UNIQUE INDEX` |
| `idx_trials_project_id` | インデックス | `CREATE INDEX` |
| `idx_parameters_step_id` | インデックス | `CREATE INDEX` |

### Rust 側の定義

上記の接頭辞・接尾辞は `backend/src/repository/naming_conventions.rs` に定数として定義する。
制約名からテーブル接頭辞を取り除くヘルパーも同モジュールに集約しており、
テーブル名が複数形であることを前提に `{entity}s_` のみを対象とする。

`map_sqlx_error` はこの規約を前提に制約名からエンティティ名・フィールド名を復元する。
**規約から外れた名前を付けてもコンパイルは通る**ため、リネームするとエラー分類だけが静かに劣化する
（一意制約違反が `Conflict` の `field` に制約名がそのまま現れる、外部キー違反の向き判定が崩れる等）。
マイグレーションで制約名を変更する場合は `naming_conventions.rs` とそのテストも併せて更新すること。

## アンチパターン

```rust
// ❌ リポジトリ内でビジネスロジック
if name.is_empty() { return Err(Error::EmptyName); }
let project = Project::new(name, ...);

// ❌ ドメインモデルに FromRow
#[derive(FromRow)]  // ドメイン層がsqlxに依存
pub struct Project { ... }

// ❌ SQLインジェクション
let query = format!("SELECT * FROM projects WHERE name = '{}'", name);

// ❌ トランザクション管理
let mut tx = self.pool.begin().await?;

// ✅ 受け取ったモデルを保存、プレースホルダー使用、単一操作のみ
```

## チェックリスト

- [ ] portsトレイトを実装
- [ ] ビジネスロジックを含まない
- [ ] DBモデルとドメインモデルが分離
- [ ] プレースホルダー使用（SQLインジェクション対策）
- [ ] UPSERT（ON CONFLICT）で冪等性確保
- [ ] sqlx エラーは `map_sqlx_error` で変換する（`RepositoryError::Internal` に畳まない）
- [ ] テーブル名は複数形の snake_case、エンティティ名はその単数形
- [ ] 主キー・一意制約・外部キーは PostgreSQL のデフォルト名をリネームしない
- [ ] インデックスは `idx_{table}_{columns}` で明示的に命名し、一意性は `CREATE UNIQUE INDEX` で表現する

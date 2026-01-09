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
            .map_err(|e| RepositoryError::Internal { message: e.to_string() })
    }

    // find_all, save, exists_by_name, ...
}
```

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

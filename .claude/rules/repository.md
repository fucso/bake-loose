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

## リポジトリ実装

```rust
// src/repository/project_repo.rs

pub struct PgProjectRepository {
    pool: PgPool,
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        sqlx::query(r#"
            INSERT INTO projects (id, name, status, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name, status = EXCLUDED.status, updated_at = NOW()
        "#)
        .bind(project.id().0)
        .bind(project.name())
        .bind(project.status().to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Internal { message: e.to_string() })?;
        Ok(())
    }

    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.map(Project::from))
            .map_err(|e| RepositoryError::Internal { message: e.to_string() })
    }
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

# Ports Layer

Ports層はドメイン層とインフラ層の境界を定義し、リポジトリトレイト（インターフェース）を定義する。

## 基本原則

- **依存**: domain層にのみ依存
- **禁止**: 外部クレート（sqlx, axum等）、実装詳細（SQL、特定のDB）
- **役割**: Dependency Inversion（依存性逆転）の実現

```
use_case層 → ports層（トレイト）→ domain層
                ↑ 実装
            repository層
```

## ファイル配置

```
backend/src/ports/
├── project_repository.rs
├── trial_repository.rs
├── ...
├── unit_of_work.rs
└── error.rs
```

## リポジトリトレイト

```rust
// src/ports/project_repository.rs

use crate::domain::models::project::{Project, ProjectId};

#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;
    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError>;
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &ProjectId) -> Result<(), RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
}
```

## エラー型

```rust
// src/ports/error.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    NotFound { entity: String, id: String },
    Conflict { entity: String, field: String },
    Connection,
    Internal { message: String },
}
```

## UnitOfWork パターン

複数リポジトリを跨ぐトランザクション管理に使用。

```rust
// src/ports/unit_of_work.rs

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type ProjectRepo: ProjectRepository;
    type TrialRepo: TrialRepository;
    // ...

    /// リポジトリを取得（呼び出すたびに新しいインスタンスを返す）
    fn project_repository(&mut self) -> Self::ProjectRepo;
    fn trial_repository(&mut self) -> Self::TrialRepo;
    // ...

    /// トランザクションを開始（書き込み操作前に呼び出す）
    async fn begin(&mut self) -> Result<(), RepositoryError>;
    async fn commit(&mut self) -> Result<(), RepositoryError>;
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}
```

**設計ポイント**:
- `project_repository()` は毎回新しいインスタンスを返す（Rust の借用ルール対応）
- トランザクション状態は UnitOfWork 内で管理し、リポジトリ間で共有される
- 読み取り専用の場合は `begin()` 不要（pool を直接使用）

## 命名規則

| 操作 | パターン |
|------|---------|
| 単一取得 | `find_by_{field}` |
| 複数取得 | `find_all`, `find_by_{field}` |
| 存在チェック | `exists_by_{field}` |
| 保存 | `save` |
| 削除 | `delete` |

## ソート機能

ソート機能は以下のように責務を分離する:

```
ports/sort.rs              → 汎用型（モデル非依存）
  SortDirection, SortColumn trait, Sort<C>

ports/{model}_repository.rs → ソート可能フィールドの定義
  {Model}SortColumn enum

repository/models/{model}_row.rs → DBカラム名へのマッピング
  impl SortColumn for {Model}SortColumn
```

**ports層での定義:**

```rust
// src/ports/sort.rs - 汎用型
pub trait SortColumn: Send + Sync + Copy {
    fn as_sql_column(&self) -> &'static str;
}

pub struct Sort<C: SortColumn> {
    pub column: C,
    pub direction: SortDirection,
}

// src/ports/project_repository.rs - ソート可能フィールド
pub enum ProjectSortColumn {
    Name,
    CreatedAt,
    UpdatedAt,
}

pub type ProjectSort = Sort<ProjectSortColumn>;
```

**repository層での実装は repository.md を参照。**

## アンチパターン

```rust
// ❌ SQL固有の構文
async fn find_by_sql(&self, where_clause: &str) -> Result<Vec<Project>, RepositoryError>;

// ❌ DB固有の型
use sqlx::PgPool;
fn with_pool(pool: PgPool) -> Self;

// ❌ ビジネスロジック
async fn save_if_active(&self, project: &Project) -> Result<(), RepositoryError>;

// ✅ 抽象的なメソッド、ドメインモデルの型のみ
async fn find_by_status(&self, status: ProjectStatus) -> Result<Vec<Project>, RepositoryError>;
```

## チェックリスト

- [ ] domain層にのみ依存
- [ ] 外部クレートへの依存がない
- [ ] `async_trait` + `Send + Sync`バウンド
- [ ] すべてのメソッドが`Result`型を返す
- [ ] ドメインモデルの型のみを使用

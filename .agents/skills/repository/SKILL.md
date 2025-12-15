# Repository Layer Skill

## 概要

リポジトリ層は ports 層で定義されたトレイトの具体実装を担当する。
データベース（PostgreSQL）へのアクセス、SQL の発行、ドメインモデルとDBモデル間の変換を行う。

---

## 基本原則

### 依存の方向

- リポジトリ層は **ports 層のトレイト** を実装する
- **domain 層のモデル** を入出力として使用する
- **infrastructure 層** のデータベース接続を利用する

```
repository → ports（トレイト実装）
repository → domain（モデルの入出力）
repository → infrastructure（DB接続）
```

### 責務の範囲

| やること | やらないこと |
|---------|-------------|
| ports トレイトの実装 | ビジネスロジック |
| SQL の発行 | バリデーション |
| ドメインモデル ↔ DBモデルの変換 | 複数リポジトリのオーケストレーション |
| データ型に応じた格納 | トランザクション境界の管理 |

**注意:** トランザクション管理はユースケース層の責務。リポジトリは単一の操作を提供する。

---

## ファイル配置

```
backend/src/repository/
├── project_repo.rs       # ProjectRepository トレイトの実装
├── trial_repo.rs         # TrialRepository トレイトの実装
├── feedback_repo.rs      # FeedbackRepository トレイトの実装
└── models/               # DBモデル（行マッピング用）
    ├── project_row.rs
    ├── trial_row.rs
    └── feedback_row.rs
```

**命名規則:**
- リポジトリ実装: `{entity}_repo.rs`
- DBモデル: `{entity}_row.rs`

---

## 技術スタック

### PostgreSQL / SQLx

| ツール | 用途 |
|--------|------|
| PostgreSQL | データベース |
| SQLx | 非同期 SQL クライアント（コンパイル時クエリ検証） |

---

## DBモデル（Row 構造体）

### DBモデルとは

DBモデルは **SQLx のクエリ結果をマッピングするための構造体**。
ドメインモデルとは別に定義し、変換処理を経由してドメインモデルに変換する。

### なぜ分離するのか

| 観点 | ドメインモデル | DBモデル |
|------|--------------|---------|
| 目的 | ビジネス概念の表現 | DB行のマッピング |
| フィールド | ビジネス上必要なもの | カラムと1:1対応 |
| 型 | NewType（ProjectId等） | プリミティブ（Uuid等） |
| 依存 | なし（純粋） | sqlx, chrono 等 |

### 実装例

```rust
// backend/src/repository/models/project_row.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// データベースの projects テーブルに対応する行構造体
#[derive(Debug, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 変換トレイトの実装

ドメインモデルとDBモデル間の変換を実装する：

```rust
use crate::domain::models::project::{Project, ProjectId, ProjectStatus};

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Project::reconstruct(
            ProjectId(row.id),
            row.name,
            row.description,
            row.goal,
            row.status.parse().unwrap_or_default(),
        )
    }
}

impl From<&Project> for ProjectRow {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id().0,
            name: project.name().to_string(),
            description: project.description().map(|s| s.to_string()),
            goal: project.goal().map(|s| s.to_string()),
            status: project.status().to_string(),
            created_at: Utc::now(), // 保存時に更新
            updated_at: Utc::now(),
        }
    }
}
```

---

## リポジトリ実装

### 基本構造

```rust
// backend/src/repository/project_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::project_repository::{ProjectRepository, RepositoryError};
use crate::repository::models::project_row::ProjectRow;

/// ProjectRepository トレイトの PostgreSQL 実装
pub struct PgProjectRepository {
    pool: PgPool,
}

impl PgProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, description, goal, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                goal = EXCLUDED.goal,
                status = EXCLUDED.status,
                updated_at = NOW()
            "#,
        )
        .bind(project.id().0)
        .bind(project.name())
        .bind(project.description())
        .bind(project.goal())
        .bind(project.status().to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        let row = sqlx::query_as::<_, ProjectRow>(
            "SELECT * FROM projects WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(row.map(Project::from))
    }

    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            "SELECT * FROM projects ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Project::from).collect())
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM projects WHERE name = $1",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(count.0 > 0)
    }

    async fn delete(&self, id: &ProjectId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}
```

---

## エラーハンドリング

### リポジトリエラー

リポジトリ層のエラーは **ports 層で定義された `RepositoryError` をそのまま使用する**（再定義しない）：

```rust
// ports 層の定義を使用
use crate::ports::error::RepositoryError;
```

ports 層での定義（参照）:

```rust
/// リポジトリ操作で発生するエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    /// データが見つからない（期待していた場合）
    NotFound { entity: String, id: String },

    /// 一意性制約違反（重複）
    Conflict { entity: String, field: String },

    /// 接続エラー
    Connection,

    /// その他の内部エラー
    Internal { message: String },
}
```

### SQLx エラーの変換

```rust
impl From<sqlx::Error> for RepositoryError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound {
                entity: "unknown".to_string(),
                id: "unknown".to_string(),
            },
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => {
                RepositoryError::Connection
            }
            _ => RepositoryError::Internal {
                message: e.to_string(),
            },
        }
    }
}
```

---

## アンチパターン

### NG: ビジネスロジックの実装

```rust
// ❌ リポジトリ内でビジネスロジックを実行
impl ProjectRepository for PgProjectRepository {
    async fn create(&self, name: &str) -> Result<Project, Error> {
        // リポジトリ内でバリデーション（NG）
        if name.is_empty() {
            return Err(Error::EmptyName);
        }
        // リポジトリ内でモデル生成（NG）
        let project = Project::new(name.to_string(), None, None);
        // ...
    }
}

// ✅ リポジトリは永続化のみを担当
impl ProjectRepository for PgProjectRepository {
    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        // 受け取ったドメインモデルをそのまま保存
        sqlx::query(/* ... */)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
// バリデーションとモデル生成はドメイン層・ユースケース層の責務
```

### NG: ドメインモデルを直接 FromRow で使う

```rust
// ❌ ドメインモデルに sqlx の derive を付ける
use sqlx::FromRow;

#[derive(FromRow)]  // ドメイン層が sqlx に依存してしまう
pub struct Project {
    pub id: Uuid,  // NewType ではなく生の Uuid
    // ...
}

// ✅ DBモデル（Row）を別に定義して変換
#[derive(FromRow)]
pub struct ProjectRow { /* ... */ }

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self { /* ... */ }
}
```

### NG: SQL インジェクションの危険がある書き方

```rust
// ❌ 文字列連結でクエリを構築
async fn find_by_name(&self, name: &str) -> Result<Option<Project>, Error> {
    let query = format!("SELECT * FROM projects WHERE name = '{}'", name);
    sqlx::query_as(&query).fetch_optional(&self.pool).await
}

// ✅ プレースホルダーを使用
async fn find_by_name(&self, name: &str) -> Result<Option<Project>, Error> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE name = $1")
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Project::from))
}
```

### NG: トランザクション管理をリポジトリで行う

```rust
// ❌ リポジトリ内でトランザクションを管理
impl ProjectRepository for PgProjectRepository {
    async fn create_with_trial(&self, project: &Project, trial: &Trial) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        // project 保存
        // trial 保存
        tx.commit().await?;
        Ok(())
    }
}

// ✅ トランザクション管理はユースケース層で行う
// リポジトリは単一の操作を提供
impl ProjectRepository for PgProjectRepository {
    async fn save(&self, project: &Project) -> Result<(), Error> { /* ... */ }
}
impl TrialRepository for PgTrialRepository {
    async fn save(&self, trial: &Trial) -> Result<(), Error> { /* ... */ }
}
// ユースケース層で両方を呼び出してトランザクションを管理
```

---

## テスト

### テスト方針

リポジトリ層のテストは統合テストとして実施する（実際のDBを使用）。

### テスト用データベース

```rust
// backend/tests/common/mod.rs

use sqlx::PgPool;

pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url).await.unwrap();

    // テストごとにクリーンな状態にする
    sqlx::query("TRUNCATE projects, trials, feedbacks CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    pool
}
```

### テスト例

```rust
// backend/tests/repository/project_repo_test.rs

use crate::common::setup_test_db;

#[tokio::test]
async fn プロジェクトの保存と取得() {
    let pool = setup_test_db().await;
    let repo = PgProjectRepository::new(pool);

    // テスト用のプロジェクトを作成
    let project = Project::new(
        "テストプロジェクト".to_string(),
        Some("説明".to_string()),
        None,
    );

    // 保存
    repo.save(&project).await.unwrap();

    // 取得
    let found = repo.find_by_id(project.id()).await.unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name(), "テストプロジェクト");
    assert_eq!(found.description(), Some("説明"));
}

#[tokio::test]
async fn 存在チェック() {
    let pool = setup_test_db().await;
    let repo = PgProjectRepository::new(pool);

    // 存在しない名前
    assert!(!repo.exists_by_name("存在しない").await.unwrap());

    // プロジェクトを作成
    let project = Project::new("テスト".to_string(), None, None);
    repo.save(&project).await.unwrap();

    // 存在する名前
    assert!(repo.exists_by_name("テスト").await.unwrap());
}
```

---

## チェックリスト

リポジトリ層のコードをレビューする際は以下を確認：

### 基本原則
- [ ] ports 層のトレイトを実装している
- [ ] ビジネスロジックを含んでいない
- [ ] トランザクション管理を行っていない

### DBモデル
- [ ] ドメインモデルと分離されている
- [ ] `FromRow` derive を使用している
- [ ] ドメインモデルとの変換（From トレイト）が実装されている

### SQL
- [ ] プレースホルダーを使用している（SQL インジェクション対策）
- [ ] UPSERT（ON CONFLICT）を使用して冪等性を確保している
- [ ] 適切なインデックスが定義されている

### エラーハンドリング
- [ ] sqlx::Error を RepositoryError に変換している
- [ ] 適切なエラー種別（NotFound, Database 等）を返している

### テスト
- [ ] 統合テストが記述されている
- [ ] テスト用データベースを使用している
- [ ] テストごとにデータをクリーンアップしている

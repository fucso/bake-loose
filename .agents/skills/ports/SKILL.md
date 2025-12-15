# Ports Layer Skill

## 概要

Ports層はドメイン層とインフラ層の境界を定義する層であり、リポジトリトレイト（インターフェース）を定義する。
このスキルはPorts層の設計・実装ルールを定義する。

---

## 基本原則

### 境界としての役割

Ports層は **Dependency Inversion Principle（依存性逆転の原則）** を実現するための境界である。

```
┌─────────────────────────────────────────────────────────────┐
│  use_case層                                                  │
│    ↓ 依存             依存                                    │
│  ports層（トレイト定義） →  domain層                            │
│    ↑ 実装                                                    │
│  repository層（具体実装）                                      │
└─────────────────────────────────────────────────────────────┘
```

- **use_case層** は ports層のトレイトに依存する（抽象に依存）
- **repository層** は ports層のトレイトを実装する（具体実装）
- これにより、use_case層は永続化の詳細を知らずに済む

### 依存の方向

- **Ports層はドメイン層にのみ依存する**
- use_case層から参照される
- repository層から実装される
- 外部クレート（sqlx, axum 等）への依存禁止

### 抽象化の原則

- 実装詳細（SQL、特定のDB、ファイルパス等）を含めない
- ドメインモデルの型のみを使用する
- メソッドシグネチャは「何ができるか」を表し、「どうやるか」は含めない

---

## ファイル配置

```
backend/src/ports/
├── project_repository.rs
├── trial_repository.rs
└── feedback_repository.rs
```

**1エンティティ1ファイル** の原則を採用する。これにより：
- トレイトの責務が明確になる
- 変更影響範囲が限定される
- テスト用モック実装が作りやすくなる

---

## トレイト定義ルール

### 基本構造

```rust
// backend/src/ports/project_repository.rs

use crate::domain::models::project::{Project, ProjectId};

/// プロジェクトの永続化を担当するリポジトリトレイト
#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    /// IDでプロジェクトを取得
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;

    /// すべてのプロジェクトを取得
    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError>;

    /// プロジェクトを保存（作成・更新）
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;

    /// プロジェクトを削除
    async fn delete(&self, id: &ProjectId) -> Result<(), RepositoryError>;
}
```

### async_trait の使用

Rustのトレイトでは非同期メソッドを直接定義できないため、`async_trait` クレートを使用する

**`Send + Sync` バウンドの理由：**
- 非同期ランタイム（tokio）でスレッド間で安全に共有するため
- DIコンテナで `Arc<dyn Repository>` として保持するため

### Result型の使用

すべてのメソッドは `Result` 型を返す：

```rust
// 永続化エラーの可能性があるため
async fn save(&self, project: &Project) -> Result<(), RepositoryError>;

// 見つからない場合は None（エラーではない）
async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;
```

---

## エラー型の定義

### RepositoryError

Ports層でリポジトリ共通のエラー型を定義する：

```rust
// backend/src/ports/error.rs

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

**エラー設計のポイント：**

| 項目 | 方針 |
|------|------|
| 具体的すぎない | `SqlxError` などDB固有のエラーは含めない |
| 抽象的すぎない | エラーの種類を区別できる程度の詳細度 |
| use_case層で判断可能 | エラーの種類に応じた処理分岐ができる |

---

## メソッドパターン

### 基本CRUD

```rust
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // Create / Update（upsert パターン）
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;

    // Read（単一）
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;

    // Read（複数）
    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError>;

    // Delete
    async fn delete(&self, id: &ProjectId) -> Result<(), RepositoryError>;
}
```

### 存在チェック用メソッド

重複チェックなど、ユースケース層で必要となる存在確認用メソッドを定義する：

```rust
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // 基本CRUD...

    /// 指定した名前のプロジェクトが存在するか確認
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;

    /// 指定したIDを除いて、名前が重複するか確認
    async fn exists_by_name_excluding_id(
        &self,
        name: &str,
        exclude_id: &ProjectId,
    ) -> Result<bool, RepositoryError>;
}
```

**存在チェックの設計理由：**
- ドメイン層は純粋関数であり、DB問い合わせができない
- 重複チェックなどの検証はユースケース層で行う
- ユースケース層がリポジトリ経由で存在確認を行うためのメソッドが必要

### 検索・フィルタリング

検索・フィルタリングは **ドメインモデルのフィールド** をベースに定義する。
DBスキーマとドメインモデルに差分がある場合（カラム名の違い、正規化の違いなど）は、repository層の実装で吸収する。

```rust
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // 基本メソッド...

    /// ステータスでフィルタリング（ドメインの ProjectStatus を使用）
    async fn find_by_status(&self, status: ProjectStatus) -> Result<Vec<Project>, RepositoryError>;

    /// 名前で部分一致検索（ドメインの name フィールドに対応）
    async fn search_by_name(&self, query: &str) -> Result<Vec<Project>, RepositoryError>;
}
```

**ポイント:**
- トレイトのシグネチャはドメインモデルの型・フィールド名を使用
- DBスキーマの詳細（カラム名、テーブル構造）はports層に漏らさない
- スキーマとドメインのマッピングはrepository層の責務

### 関連エンティティの取得

```rust
#[async_trait]
pub trait TrialRepository: Send + Sync {
    // 基本メソッド...

    /// プロジェクトに紐づくすべての試行を取得
    async fn find_by_project_id(&self, project_id: &ProjectId) -> Result<Vec<Trial>, RepositoryError>;
}

#[async_trait]
pub trait FeedbackRepository: Send + Sync {
    // 基本メソッド...

    /// 試行に紐づくすべてのフィードバックを取得
    async fn find_by_trial_id(&self, trial_id: &TrialId) -> Result<Vec<Feedback>, RepositoryError>;
}
```

---

## 命名規則

### トレイト名

| パターン | 例 |
|---------|-----|
| `{Entity}Repository` | `ProjectRepository`, `TrialRepository` |

### メソッド名

| 操作 | パターン | 例 |
|------|---------|-----|
| 単一取得 | `find_by_{field}` | `find_by_id`, `find_by_name` |
| 複数取得 | `find_all`, `find_by_{field}` | `find_all`, `find_by_status` |
| 存在チェック | `exists_by_{field}` | `exists_by_name` |
| 保存 | `save` | `save` |
| 削除 | `delete` | `delete` |
| 検索 | `search_by_{field}` | `search_by_name` |

---

## アンチパターン

### NG: 実装詳細の漏洩

```rust
// ❌ SQL固有の構文が漏れている
pub trait ProjectRepository {
    async fn find_by_sql(&self, where_clause: &str) -> Result<Vec<Project>, RepositoryError>;
}

// ✅ 抽象的なメソッドを定義
pub trait ProjectRepository {
    async fn find_by_status(&self, status: ProjectStatus) -> Result<Vec<Project>, RepositoryError>;
    async fn search_by_name(&self, query: &str) -> Result<Vec<Project>, RepositoryError>;
}
```

### NG: DB固有の型の使用

```rust
// ❌ sqlx の型を使用
use sqlx::PgPool;

pub trait ProjectRepository {
    fn with_pool(pool: PgPool) -> Self;
}

// ✅ ドメインモデルの型のみを使用
use crate::domain::models::project::{Project, ProjectId};

pub trait ProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;
}
```

### NG: 外部クレートへの依存

```rust
// ❌ ports層で sqlx に依存
use sqlx::Error as SqlxError;

pub enum RepositoryError {
    Sqlx(SqlxError),  // NG: DB固有のエラーを含めない
}

// ✅ 抽象的なエラー型を定義
pub enum RepositoryError {
    NotFound { entity: String, id: String },
    Connection,
    Internal { message: String },
}
```

### NG: ビジネスロジックの記述

```rust
// ❌ リポジトリトレイトにビジネスロジックを含める
pub trait ProjectRepository {
    /// アーカイブ済みでなければ保存
    async fn save_if_active(&self, project: &Project) -> Result<(), RepositoryError>;
}

// ✅ 単純なCRUD操作のみを定義
pub trait ProjectRepository {
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
}
// ビジネスロジック（アーカイブ済みかどうか）はuse_case層で判断
```

### NG: リポジトリトレイトでのトランザクション管理

```rust
// ❌ 個々のリポジトリトレイトでトランザクションを定義
pub trait ProjectRepository {
    async fn begin_transaction(&self) -> Result<Transaction, RepositoryError>;
    async fn commit(&self, tx: Transaction) -> Result<(), RepositoryError>;
}

// ✅ トランザクション境界の管理は use_case 層の責務
// ports 層ではトランザクション管理用の別トレイトを定義することを検討
// 例: UnitOfWork パターン
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> Result<(), RepositoryError>;
    async fn commit(&self) -> Result<(), RepositoryError>;
    async fn rollback(&self) -> Result<(), RepositoryError>;
}
```

**トランザクション管理の方針:**
- トランザクション境界の**決定**は use_case 層の責務
- ただし use_case 層が DB 固有の知識を持つことは避けるため、トランザクション操作は ports 層のトレイト経由で抽象化する
- 個々のリポジトリトレイトにトランザクション操作を含めない
- 必要な場合は UnitOfWork などの専用トレイトを ports 層で定義

---

## ユースケース層での使用例

Ports層のトレイトはユースケース層で以下のように使用される：

```rust
// backend/src/use_case/project/create_project.rs

use crate::domain::actions::project::create_project;
use crate::ports::project_repository::ProjectRepository;

pub struct CreateProjectUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> CreateProjectUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: create_project::Command) -> Result<Project, UseCaseError> {
        // 重複チェック（ports経由）
        if self.repository.exists_by_name(&command.name).await? {
            return Err(UseCaseError::DuplicateName);
        }

        // ドメインアクション実行
        let project = create_project::run(command)?;

        // 永続化（ports経由）
        self.repository.save(&project).await?;

        Ok(project)
    }
}
```

---

## テスト

### テスト用モック実装

Ports層のトレイトはテスト用のモック実装を作成しやすい：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// テスト用のインメモリリポジトリ
    struct MockProjectRepository {
        storage: Mutex<HashMap<ProjectId, Project>>,
    }

    impl MockProjectRepository {
        fn new() -> Self {
            Self {
                storage: Mutex::new(HashMap::new()),
            }
        }

        fn with_projects(projects: Vec<Project>) -> Self {
            let mut storage = HashMap::new();
            for project in projects {
                storage.insert(project.id().clone(), project);
            }
            Self {
                storage: Mutex::new(storage),
            }
        }
    }

    #[async_trait]
    impl ProjectRepository for MockProjectRepository {
        async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
            let storage = self.storage.lock().unwrap();
            Ok(storage.get(id).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Project>, RepositoryError> {
            let storage = self.storage.lock().unwrap();
            Ok(storage.values().cloned().collect())
        }

        async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
            let mut storage = self.storage.lock().unwrap();
            storage.insert(project.id().clone(), project.clone());
            Ok(())
        }

        async fn delete(&self, id: &ProjectId) -> Result<(), RepositoryError> {
            let mut storage = self.storage.lock().unwrap();
            storage.remove(id);
            Ok(())
        }

        async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
            let storage = self.storage.lock().unwrap();
            Ok(storage.values().any(|p| p.name() == name))
        }
    }

    #[tokio::test]
    async fn プロジェクトを保存して取得できる() {
        let repo = MockProjectRepository::new();
        let project = Project::new("テスト".to_string(), None, None);
        let id = project.id().clone();

        repo.save(&project).await.unwrap();
        let found = repo.find_by_id(&id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "テスト");
    }
}
```

### mockallクレートの使用

より高度なモックが必要な場合は `mockall` クレートを使用できる：

```rust
use mockall::automock;

#[automock]
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
    // ...
}

// テストでの使用
#[tokio::test]
async fn 重複名でエラーになる() {
    let mut mock_repo = MockProjectRepository::new();
    mock_repo
        .expect_exists_by_name()
        .with(eq("既存の名前"))
        .returning(|_| Ok(true));

    let use_case = CreateProjectUseCase::new(mock_repo);
    let result = use_case.execute(Command { name: "既存の名前".to_string() }).await;

    assert!(matches!(result, Err(UseCaseError::DuplicateName)));
}
```

---

## チェックリスト

Ports層のコードをレビューする際は以下を確認:

### 基本原則
- [ ] ドメイン層にのみ依存している
- [ ] 外部クレート（sqlx, axum 等）への依存がない
- [ ] 実装詳細（SQL、特定のDB等）を含めていない

### トレイト定義
- [ ] `async_trait` マクロを使用している
- [ ] `Send + Sync` バウンドを付けている
- [ ] すべてのメソッドが `Result` 型を返している
- [ ] ドメインモデルの型のみを使用している

### メソッド設計
- [ ] 基本CRUD（save, find_by_id, find_all, delete）が定義されている
- [ ] 存在チェック用メソッド（exists_by_*）が必要に応じて定義されている
- [ ] 関連エンティティ取得用メソッド（find_by_*_id）が必要に応じて定義されている

### エラー型
- [ ] RepositoryError が抽象的なエラー型として定義されている
- [ ] DB固有のエラー型を含めていない
- [ ] use_case層で判断可能な程度の詳細度である

### 命名規則
- [ ] トレイト名が `{Entity}Repository` パターンに従っている
- [ ] メソッド名が規約（find_by_*, exists_by_*, save, delete等）に従っている

### ファイル配置
- [ ] `src/ports/` 配下に配置されている
- [ ] 1エンティティ1ファイルになっている

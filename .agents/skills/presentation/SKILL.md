# Presentation Layer Skill

## 概要

プレゼンテーション層はアプリケーションの最外層であり、外部インターフェース（GraphQL）を担当する。
このスキルはGraphQLリゾルバー・スキーマの設計・実装ルールを定義する。

---

## 基本原則

### 役割

- **薄く保つ**: プレゼンテーション層はユースケースを呼び出すだけの薄い層
- **外部インターフェースの提供**: GraphQL スキーマの定義とリゾルバーの実装
- **エラー変換**: 各層のエラー（domain, use_case, repository）をユーザー向けメッセージに変換
- **入力の整形**: GraphQL 入力型からユースケースが期待するコマンド型への変換

### 依存の方向

```
presentation → use_case → ports → domain
                            ↓
                        repository → infrastructure
```

- プレゼンテーション層は **ユースケース層のみに依存** する
- ドメイン層のモデルは GraphQL の型定義で参照可能（読み取りのみ）
- リポジトリ層や infrastructure 層には直接依存しない

---

## ファイル配置

```
backend/src/presentation/
├── graphql/
│   ├── schema.rs         # スキーマ全体の組み立て
│   ├── types/
│   │   ├── project.rs    # Project の GraphQL 型定義
│   │   ├── trial.rs      # Trial の GraphQL 型定義
│   │   └── feedback.rs   # Feedback の GraphQL 型定義
│   ├── query/
│   │   ├── project.rs    # Project 関連のクエリ
│   │   └── ...
│   ├── mutation/
│   │   ├── project.rs    # Project 関連のミューテーション
│   │   └── ...
│   └── error.rs          # GraphQL エラー型・エラー変換
└── ...
```

---

## Async-GraphQL の使用規約

### スキーマの組み立て

```rust
// backend/src/presentation/graphql/schema.rs

use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::presentation::graphql::mutation::{ProjectMutation, TrialMutation, FeedbackMutation};
use crate::presentation::graphql::query::{ProjectQuery, TrialQuery, FeedbackQuery};

#[derive(MergedObject, Default)]
pub struct QueryRoot(ProjectQuery, TrialQuery, FeedbackQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(ProjectMutation, TrialMutation, FeedbackMutation);

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(/* 依存性 */) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    // .data(...)  // ユースケースやリポジトリの注入
    .finish()
}
```

### GraphQL 型の定義

ドメインモデルを GraphQL 型として公開する際は、専用の型を定義する：

```rust
// backend/src/presentation/graphql/types/project.rs

use async_graphql::{Object, SimpleObject, ID};
use crate::domain::models::project::{Project as DomainProject, ProjectStatus as DomainStatus};

/// GraphQL の Project 型
pub struct Project(pub DomainProject);

#[Object]
impl Project {
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    async fn name(&self) -> &str {
        self.0.name()
    }

    async fn description(&self) -> Option<&str> {
        self.0.description()
    }

    async fn status(&self) -> ProjectStatus {
        self.0.status().into()
    }

    // 関連エンティティの解決（DataLoader 使用推奨）
    async fn trials(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<Trial>> {
        // DataLoader 経由で取得
        // ...
    }
}

/// GraphQL の ProjectStatus 列挙型
#[derive(SimpleObject, Clone, Copy)]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl From<DomainStatus> for ProjectStatus {
    fn from(status: DomainStatus) -> Self {
        match status {
            DomainStatus::Active => ProjectStatus::Active,
            DomainStatus::Archived => ProjectStatus::Archived,
        }
    }
}
```

### ドメインモデルを直接公開しない理由

- GraphQL スキーマの変更がドメインモデルの変更を強制しない
- ドメインモデルに GraphQL 固有のアノテーションを付けずに済む
- フィールドの公開範囲を GraphQL 層でコントロールできる

---

## リゾルバーの実装

### クエリリゾルバー

```rust
// backend/src/presentation/graphql/query/project.rs

use async_graphql::{Context, Object, Result, ID};
use crate::use_case::project::get_project::GetProjectUseCase;
use crate::presentation::graphql::types::Project;
use crate::presentation::graphql::error::GraphQLError;

#[derive(Default)]
pub struct ProjectQuery;

#[Object]
impl ProjectQuery {
    /// 指定した ID のプロジェクトを取得
    async fn project(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Project>> {
        let use_case = ctx.data::<GetProjectUseCase>()?;
        let project_id = parse_project_id(&id)?;

        let result = use_case.execute(project_id).await;

        match result {
            Ok(Some(project)) => Ok(Some(Project(project))),
            Ok(None) => Ok(None),
            Err(e) => Err(GraphQLError::from(e).into()),
        }
    }

    /// すべてのプロジェクトを取得
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let use_case = ctx.data::<ListProjectsUseCase>()?;

        let result = use_case.execute().await;

        match result {
            Ok(projects) => Ok(projects.into_iter().map(Project).collect()),
            Err(e) => Err(GraphQLError::from(e).into()),
        }
    }
}
```

### ミューテーションリゾルバー

```rust
// backend/src/presentation/graphql/mutation/project.rs

use async_graphql::{Context, InputObject, Object, Result, ID};
use crate::use_case::project::create_project::{CreateProjectUseCase, Command};
use crate::presentation::graphql::types::Project;
use crate::presentation::graphql::error::GraphQLError;

/// プロジェクト作成の入力型
#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub goal: Option<String>,
}

impl From<CreateProjectInput> for Command {
    fn from(input: CreateProjectInput) -> Self {
        Command {
            name: input.name,
            description: input.description,
            goal: input.goal,
        }
    }
}

#[derive(Default)]
pub struct ProjectMutation;

#[Object]
impl ProjectMutation {
    /// 新しいプロジェクトを作成
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let use_case = ctx.data::<CreateProjectUseCase>()?;

        let result = use_case.execute(input.into()).await;

        match result {
            Ok(project) => Ok(Project(project)),
            Err(e) => Err(GraphQLError::from(e).into()),
        }
    }
}
```

### リゾルバーの責務（重要）

| やること | やらないこと |
|---------|-------------|
| ユースケースの呼び出し | ビジネスロジックの実装 |
| 入力型からコマンド型への変換 | バリデーション（ドメイン層の責務） |
| エラーの変換 | 直接的な DB アクセス |
| GraphQL 型への変換 | 複雑な条件分岐 |

---

## エラー変換

### 設計原則

**すべての層（domain, use_case, repository）のエラーは種類のみを定義し、プレゼンテーション層でユーザー向けメッセージに変換する。**

これにより：
- 各層がプレゼンテーションの関心事（メッセージ、多言語化等）から独立
- エラーメッセージの変更がプレゼンテーション層で完結
- 同じエラー種類を異なるコンテキストで異なるメッセージに変換可能

### GraphQL エラー型の設計

```rust
// backend/src/presentation/graphql/error.rs

use async_graphql::{Error as GqlError, ErrorExtensions};

/// GraphQL エラー型
#[derive(Debug)]
pub struct GraphQLError {
    message: String,
    code: String,
    extensions: Option<serde_json::Value>,
}

impl GraphQLError {
    pub fn new(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            extensions: None,
        }
    }

    pub fn with_extensions(mut self, extensions: serde_json::Value) -> Self {
        self.extensions = Some(extensions);
        self
    }
}

impl From<GraphQLError> for GqlError {
    fn from(e: GraphQLError) -> Self {
        GqlError::new(e.message).extend_with(|_, ext| {
            ext.set("code", e.code);
        })
    }
}
```

### 各層のエラー変換例

各層のエラーは種類のみを定義し、`From` トレイトで `GraphQLError` に変換する。

```rust
// === domain 層のエラー変換 ===
impl From<create_project::Error> for GraphQLError {
    fn from(e: create_project::Error) -> Self {
        match e {
            create_project::Error::EmptyName => {
                GraphQLError::new("プロジェクト名を入力してください", "VALIDATION_ERROR")
            }
            create_project::Error::NameTooLong { max, actual } => {
                GraphQLError::new(
                    format!("プロジェクト名は{}文字以内で入力してください（現在{}文字）", max, actual),
                    "VALIDATION_ERROR",
                )
            }
        }
    }
}

// === use_case 層のエラー変換 ===
impl From<UseCaseError> for GraphQLError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound { entity, id } => {
                GraphQLError::new(format!("{}が見つかりません", entity), "NOT_FOUND")
            }
            UseCaseError::Domain(domain_error) => domain_error.into(),
            UseCaseError::Repository(repo_error) => repo_error.into(),
            // ... 他のバリアント
        }
    }
}

// === repository 層のエラー変換 ===
impl From<RepositoryError> for GraphQLError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::Connection => {
                // 内部エラーの詳細は露出しない
                GraphQLError::new(
                    "システムエラーが発生しました。しばらく経ってから再度お試しください",
                    "INTERNAL_ERROR",
                )
            }
            // ... 他のバリアント
        }
    }
}
```

### エラーコードの規約

| コード | 説明 | 例 |
|--------|------|-----|
| `VALIDATION_ERROR` | 入力値のバリデーションエラー | 空文字、文字数超過 |
| `BUSINESS_RULE_VIOLATION` | ビジネスルール違反 | アーカイブ済みへの操作 |
| `NOT_FOUND` | リソースが見つからない | 存在しない ID |
| `DUPLICATE_ERROR` | 重複エラー | 同名のプロジェクト |
| `UNAUTHORIZED` | 認証エラー | 未ログイン |
| `FORBIDDEN` | 認可エラー | 権限不足 |
| `INTERNAL_ERROR` | 内部エラー | DB エラー |

---

## 入力型の設計

### 入力型の命名規則

| 操作 | 入力型名 | 例 |
|------|---------|-----|
| 作成 | `Create*Input` | `CreateProjectInput` |
| 更新 | `Update*Input` | `UpdateProjectInput` |
| 削除 | （ID のみ） | - |
| フィルタ | `*FilterInput` | `ProjectFilterInput` |

### 入力型からコマンドへの変換

```rust
#[derive(InputObject)]
pub struct UpdateProjectNameInput {
    pub id: ID,
    pub name: String,
}

/// InputObject からユースケースの Command への変換
impl TryFrom<UpdateProjectNameInput> for (ProjectId, update_project_name::Command) {
    type Error = GraphQLError;

    fn try_from(input: UpdateProjectNameInput) -> Result<Self, Self::Error> {
        let id = parse_project_id(&input.id)?;
        let command = update_project_name::Command {
            new_name: input.name,
        };
        Ok((id, command))
    }
}
```

---

## 関連エンティティの取得

### N+1 問題への対処

関連エンティティ（例: Project に紐づく Trial 一覧）の取得は **use_case 層** の責務。
presentation 層は repository に直接アクセスしない。

```rust
// ✅ ユースケース経由で関連エンティティを取得
#[Object]
impl Project {
    async fn trials(&self, ctx: &Context<'_>) -> Result<Vec<Trial>> {
        let use_case = ctx.data::<GetTrialsByProjectUseCase>()?;
        let trials = use_case.execute(self.0.id().clone()).await?;
        Ok(trials.into_iter().map(Trial).collect())
    }
}

// ❌ presentation 層から repository を直接参照
#[Object]
impl Project {
    async fn trials(&self, ctx: &Context<'_>) -> Result<Vec<Trial>> {
        let repo = ctx.data::<TrialRepository>()?;  // ❌ アーキテクチャ違反
        let trials = repo.find_by_project_id(self.0.id()).await?;
        Ok(trials.into_iter().map(Trial).collect())
    }
}
```

N+1 問題の解決（バッチ取得など）は use_case 層と repository 層で対処する。

---

## アンチパターン

### NG: リゾルバーでビジネスロジックを実装

```rust
// ❌ リゾルバー内でビジネスロジックを実装
#[Object]
impl ProjectMutation {
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
        // ❌ バリデーションをリゾルバーで行っている
        if input.name.is_empty() {
            return Err(GqlError::new("名前は必須です"));
        }
        if input.name.len() > 100 {
            return Err(GqlError::new("名前は100文字以内です"));
        }

        // ❌ 直接リポジトリを呼んでいる
        let repo = ctx.data::<ProjectRepository>()?;
        let project = Project::new(input.name, input.description, input.goal);
        repo.save(&project).await?;

        Ok(Project(project))
    }
}

// ✅ ユースケースを呼び出すだけ
#[Object]
impl ProjectMutation {
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
        let use_case = ctx.data::<CreateProjectUseCase>()?;
        let result = use_case.execute(input.into()).await;

        match result {
            Ok(project) => Ok(Project(project)),
            Err(e) => Err(GraphQLError::from(e).into()),
        }
    }
}
```

### NG: ドメインモデルを直接 GraphQL 型として公開

```rust
// ❌ ドメインモデルに GraphQL アノテーションを直接付与
// backend/src/domain/models/project.rs
use async_graphql::SimpleObject;

#[derive(SimpleObject)]  // ❌ ドメイン層が async_graphql に依存
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    // ...
}

// ✅ プレゼンテーション層でラッパー型を定義
// backend/src/presentation/graphql/types/project.rs
pub struct Project(pub DomainProject);

#[Object]
impl Project {
    // GraphQL 固有の実装
}
```

### NG: エラーメッセージを domain / use_case / repository 層で定義

**すべての層でエラーは種類のみを定義し、メッセージはプレゼンテーション層で変換する。**

```rust
// ❌ 各層でメッセージを定義している
pub enum Error {
    EmptyName { message: String },  // ❌ メッセージを含んでいる
}

pub enum UseCaseError {
    NotFound { message: String },  // ❌ メッセージを含んでいる
}

pub enum RepositoryError {
    ConnectionFailed(String),  // ❌ メッセージを含んでいる
}

// ✅ 各層は種類のみを定義
pub enum Error {
    EmptyName,
}

pub enum UseCaseError {
    NotFound { entity: &'static str, id: String },
}

pub enum RepositoryError {
    Connection,
}

// ✅ プレゼンテーション層で変換
impl From<Error> for GraphQLError {
    fn from(e: Error) -> Self {
        match e {
            Error::EmptyName => GraphQLError::new("プロジェクト名を入力してください", "VALIDATION_ERROR"),
        }
    }
}
```

### NG: 直接リポジトリにアクセス

```rust
// ❌ リゾルバーからリポジトリを直接使用
#[Object]
impl ProjectQuery {
    async fn project(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Project>> {
        let repo = ctx.data::<ProjectRepository>()?;  // ❌
        let project = repo.find_by_id(&id).await?;
        Ok(project.map(Project))
    }
}

// ✅ ユースケース経由でアクセス
#[Object]
impl ProjectQuery {
    async fn project(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Project>> {
        let use_case = ctx.data::<GetProjectUseCase>()?;  // ✅
        let result = use_case.execute(parse_project_id(&id)?).await;
        // ...
    }
}
```

---

## テスト

### リゾルバーのテスト方針

プレゼンテーション層のテストは統合テストとして実施する。
ユースケースをモック化し、リゾルバーの動作を検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{EmptySubscription, Schema};
    use mockall::predicate::*;

    // モックユースケース
    mock! {
        pub CreateProjectUseCase {}

        impl CreateProjectUseCase {
            pub async fn execute(&self, command: Command) -> Result<DomainProject, UseCaseError>;
        }
    }

    #[tokio::test]
    async fn プロジェクト作成_正常系() {
        let mut mock_use_case = MockCreateProjectUseCase::new();
        mock_use_case
            .expect_execute()
            .returning(|cmd| {
                Ok(DomainProject::new(cmd.name, cmd.description, cmd.goal))
            });

        let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
            .data(mock_use_case)
            .finish();

        let query = r#"
            mutation {
                createProject(input: { name: "テストプロジェクト" }) {
                    name
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn プロジェクト作成_バリデーションエラー() {
        let mut mock_use_case = MockCreateProjectUseCase::new();
        mock_use_case
            .expect_execute()
            .returning(|_| {
                Err(UseCaseError::Domain(create_project::Error::EmptyName.into()))
            });

        let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
            .data(mock_use_case)
            .finish();

        let query = r#"
            mutation {
                createProject(input: { name: "" }) {
                    name
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].extensions.as_ref().unwrap().get("code").unwrap() == "VALIDATION_ERROR");
    }
}
```

### エラー変換のテスト

```rust
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn create_project_empty_name_エラー変換() {
        let domain_error = create_project::Error::EmptyName;
        let gql_error: GraphQLError = domain_error.into();

        assert_eq!(gql_error.code, "VALIDATION_ERROR");
        assert!(gql_error.message.contains("プロジェクト名"));
    }

    #[test]
    fn create_project_name_too_long_エラー変換_情報が含まれる() {
        let domain_error = create_project::Error::NameTooLong { max: 100, actual: 150 };
        let gql_error: GraphQLError = domain_error.into();

        assert_eq!(gql_error.code, "VALIDATION_ERROR");
        assert!(gql_error.message.contains("100"));
        assert!(gql_error.extensions.is_some());
    }
}
```

---

## チェックリスト

プレゼンテーション層のコードをレビューする際は以下を確認:

### 基本原則
- [ ] リゾルバーはユースケースを呼び出すだけの薄い実装になっている
- [ ] ビジネスロジックを含んでいない
- [ ] リポジトリへの直接アクセスがない

### GraphQL 型
- [ ] ドメインモデルを直接公開していない（ラッパー型を使用）
- [ ] ドメインモデルに GraphQL アノテーションを付けていない
- [ ] 入力型は命名規則に従っている（`Create*Input` 等）

### エラー変換
- [ ] 各層（domain, use_case, repository）のエラーをユーザー向けメッセージに変換している
- [ ] domain / use_case / repository 層のエラーにメッセージが含まれていない
- [ ] エラーコードが規約に従っている
- [ ] 内部エラー（DBエラー等）の詳細をクライアントに露出していない

### 依存関係
- [ ] 関連エンティティの取得はユースケース経由で行っている
- [ ] repository を直接参照していない

### テスト
- [ ] リゾルバーの統合テストが記述されている
- [ ] エラー変換のテストが記述されている

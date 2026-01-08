# Presentation Layer

プレゼンテーション層はアプリケーションの最外層。GraphQLリゾルバー・スキーマを担当。

## 基本原則

- **薄く保つ**: ユースケースを呼び出すだけ
- **依存先**: use_case層のみ（domain層のモデルは読み取りで参照可）
- **責務**: 外部インターフェース提供、エラー変換、入力整形

## ファイル配置

```
backend/src/presentation/graphql/
├── schema.rs
├── types/       # Project, Trial, ... のGraphQL型
├── query/       # クエリリゾルバー
├── mutation/    # ミューテーションリゾルバー
└── error.rs     # エラー変換
```

## GraphQL型

ドメインモデルを直接公開せず、ラッパー型を定義:

```rust
// src/presentation/graphql/types/project.rs

pub struct Project(pub DomainProject);

#[Object]
impl Project {
    async fn id(&self) -> ID { ID(self.0.id().0.to_string()) }
    async fn name(&self) -> &str { self.0.name() }
}
```

## リゾルバー

```rust
// src/presentation/graphql/mutation/project.rs

#[derive(Default)]
pub struct ProjectMutation;

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

## エラー変換

**全層のエラーは種類のみを定義し、presentation層でメッセージに変換**。

- 各 use_case のエラー型に対して `UserFacingError` trait を実装する
- query / mutation では `e.to_user_facing().extend()` を使用してエラーを変換する

```rust
// src/presentation/graphql/error.rs

impl From<create_project::Error> for GraphQLError {
    fn from(e: create_project::Error) -> Self {
        match e {
            create_project::Error::EmptyName =>
                GraphQLError::new("プロジェクト名を入力してください", "VALIDATION_ERROR"),
            create_project::Error::NameTooLong { max, actual } =>
                GraphQLError::new(format!("{}文字以内で入力してください", max), "VALIDATION_ERROR"),
        }
    }
}
```

**エラーコード規約**:
- `VALIDATION_ERROR`: 入力バリデーション
- `NOT_FOUND`: リソース不存在
- `DUPLICATE_ERROR`: 重複
- `INTERNAL_ERROR`: 内部エラー（詳細は隠す）

## アンチパターン

```rust
// ❌ リゾルバーでビジネスロジック
if input.name.is_empty() { return Err(...); }

// ❌ ドメインモデルに直接GraphQLアノテーション
#[derive(SimpleObject)]
pub struct Project { ... }  // domain層がasync_graphqlに依存

// ❌ 各層でメッセージ定義
pub enum Error { EmptyName { message: String } }

// ❌ リポジトリ直接参照
let repo = ctx.data::<ProjectRepository>()?;

// ✅ ユースケース呼び出しのみ、ラッパー型使用、エラーはpresentationで変換
```

## チェックリスト

- [ ] リゾルバーはユースケース呼び出しのみ
- [ ] ドメインモデルを直接公開していない（ラッパー型使用）
- [ ] 各層のエラーにメッセージが含まれていない
- [ ] リポジトリを直接参照していない
- [ ] 内部エラーの詳細をクライアントに露出していない

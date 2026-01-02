# Task: Presentation 実装

> Feature: [create-project](../../spec.md)
> 依存: 04-use-case

## 目的
GraphQL mutation `createProject` を実装し、クライアントからプロジェクトを作成できるようにする。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/presentation/graphql/mutation/project.rs` | 新規 | ProjectMutation リゾルバー |
| `src/presentation/graphql/mutation.rs` | 新規 | mutation モジュール定義 |
| `src/presentation/graphql/types/project.rs` | 修正 | CreateProjectInput 入力型を追加 |
| `src/presentation/graphql/error.rs` | 修正 | create_project エラーの変換を追加 |
| `src/presentation/graphql/schema.rs` | 修正 | MutationRoot を定義、AppSchema を更新 |
| `src/presentation/graphql.rs` | 修正 | mutation モジュールを追加 |

---

## 設計詳細

### GraphQL スキーマ

```graphql
input CreateProjectInput {
  name: String!
}

type Mutation {
  createProject(input: CreateProjectInput!): Project!
}
```

### CreateProjectInput

```rust
#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
}
```

### ProjectMutation リゾルバー

```rust
#[derive(Default)]
pub struct ProjectMutation;

#[Object]
impl ProjectMutation {
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        // UnitOfWork を取得
        // create_project::execute() を呼び出し
        // エラーを GraphQL エラーに変換
        // 成功時は Project ラッパー型で返す
    }
}
```

### エラー変換

`create_project::Error` から `GraphQLError` への変換を定義:

| ユースケースエラー | GraphQL エラーメッセージ | エラーコード |
|-------------------|------------------------|--------------|
| `Domain(EmptyName)` | 「プロジェクト名を入力してください」 | `VALIDATION_ERROR` |
| `Domain(NameTooLong { max, .. })` | 「{max}文字以内で入力してください」 | `VALIDATION_ERROR` |
| `DuplicateName` | 「同じ名前のプロジェクトが既に存在します」 | `DUPLICATE_ERROR` |
| `Infrastructure(_)` | 「内部エラーが発生しました」 | `INTERNAL_ERROR` |

### MutationRoot

```rust
#[derive(MergedObject, Default)]
pub struct MutationRoot(ProjectMutation);
```

### AppSchema 更新

```rust
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
```

### UnitOfWork の取得

Context から `PgUnitOfWork` を取得する方法を確認し、適切に実装する。既存の Query 実装を参考に、同様のパターンを使用する。

---

## テストケース

ファイル: `tests/graphql/projects/create.rs`（新規作成）

### ファイル構成

```
tests/
├── graphql.rs                      # (既存) mod projects; を含む
├── graphql/
│   ├── schema.rs                   # (修正) execute_graphql_with_errors を追加
│   └── projects/
│       ├── get.rs                  # (既存)
│       ├── list.rs                 # (既存)
│       └── create.rs               # (新規) createProject mutation テスト
└── fixtures/
    └── projects.sql                # (既存)
```

### テストヘルパー追加

`tests/graphql/schema.rs` に Mutation テスト用のヘルパー関数を追加:

```rust
/// GraphQL クエリを実行し、エラーを含むレスポンスを返す
pub async fn execute_graphql_with_errors(
    pool: PgPool,
    query: &str
) -> async_graphql::Response
```

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_creates_project_successfully` | 有効な入力でプロジェクトが作成される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_for_empty_name` | 空の名前で `VALIDATION_ERROR` |
| `test_returns_error_for_too_long_name` | 101文字以上の名前で `VALIDATION_ERROR` |
| `test_returns_error_for_duplicate_name` | 既存の名前で `DUPLICATE_ERROR` |

### テスト実装のポイント

- 正常系: `execute_graphql` を使用し、レスポンス JSON を完全一致で検証
  - 作成されたプロジェクトの `id` は UUID 形式であることを検証（完全一致は不可）
  - `name` は入力値と一致することを検証
- 異常系: `execute_graphql_with_errors` を使用し、エラーコードとメッセージを検証
- 重複テスト: フィクスチャで既存データを用意

---

## 完了条件

- [ ] `CreateProjectInput` 入力型が定義されている
- [ ] `ProjectMutation` リゾルバーが実装されている
- [ ] エラーが適切にユーザー向けメッセージに変換されている
- [ ] `MutationRoot` が定義され、`AppSchema` が更新されている
- [ ] `tests/graphql/schema.rs` にエラー検証用ヘルパーが追加されている
- [ ] `tests/graphql/projects/create.rs` に上記テストケースが実装されている
- [ ] `cargo test` が通る

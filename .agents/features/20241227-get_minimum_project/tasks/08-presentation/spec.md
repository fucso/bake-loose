# Task: Presentation層（GraphQL スキーマ・リゾルバー）

> Feature: [get-project](../../spec.md)
> 依存: 06-repository, 07-use-case

## 目的
GraphQL スキーマとリゾルバーを実装し、クライアントからプロジェクトを取得できるようにする。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/presentation.rs` | 新規 | presentation モジュール（graphql を再公開） |
| `backend/src/presentation/graphql.rs` | 新規 | graphql サブモジュール |
| `backend/src/presentation/graphql/schema.rs` | 新規 | スキーマ組み立て |
| `backend/src/presentation/graphql/types.rs` | 新規 | types サブモジュール（project を再公開） |
| `backend/src/presentation/graphql/types/project.rs` | 新規 | Project GraphQL型 |
| `backend/src/presentation/graphql/query.rs` | 新規 | query サブモジュール（project を再公開） |
| `backend/src/presentation/graphql/query/project.rs` | 新規 | Project クエリリゾルバー |
| `backend/src/repository/pg_unit_of_work.rs` | 新規 | UnitOfWork の PostgreSQL 実装 |
| `backend/src/lib.rs` | 修正 | presentation モジュールの公開追加 |

---

## 設計詳細

### GraphQL スキーマ

```graphql
type Query {
  project(id: ID!): Project
  projects: [Project!]!
}

type Project {
  id: ID!
  name: String!
}
```

### Project GraphQL型

ドメインモデルをラップした GraphQL 用の型:

- `Project(DomainProject)`: ドメインモデルのラッパー
- `#[Object]` マクロで GraphQL 型として定義
- `id()`: `ID` 型として返却（UUID を文字列化）
- `name()`: `&str` を返却

### クエリリゾルバー

- `project(id: ID!)`: get_project ユースケースを呼び出し
- `projects`: list_projects ユースケースを呼び出し

### リゾルバーの責務

- UnitOfWork の構築とユースケースの呼び出し
- ID のパース（String → ProjectId）
- エラーの変換（UseCase Error → GraphQL Error）

### PgUnitOfWork

UnitOfWork トレイトの PostgreSQL 実装。リゾルバーで構築してユースケースに渡す:

```rust
// src/repository/pg_unit_of_work.rs

pub struct PgUnitOfWork {
    project_repo: PgProjectRepository,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self {
            project_repo: PgProjectRepository::new(pool),
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    type ProjectRepo = PgProjectRepository;

    fn project_repository(&self) -> &Self::ProjectRepo {
        &self.project_repo
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        // 今回は読み取り専用のため何もしない
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        Ok(())
    }
}
```

### スキーマ組み立て

- `QueryRoot`: ProjectQuery をマージ
- `MutationRoot`: 今回は空（将来の拡張用）
- `AppSchema`: Schema<QueryRoot, MutationRoot, EmptySubscription>
- `build_schema(pool: PgPool) -> AppSchema`: スキーマビルダー
- スキーマのコンテキストに `PgPool` を設定し、リゾルバーで `PgUnitOfWork` を構築

---

## 完了条件

- [ ] GraphQL スキーマが定義されている
- [ ] Project 型が実装されている
- [ ] project, projects クエリが実装されている
- [ ] ドメインモデルを直接公開していない（ラッパー型を使用）
- [ ] PgUnitOfWork が実装されている
- [ ] リゾルバーが UnitOfWork 経由でユースケースを呼び出している
- [ ] `cargo check` が成功する

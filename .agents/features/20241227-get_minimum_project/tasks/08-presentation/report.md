# Task Report: Presentation層（GraphQL スキーマ・リゾルバー）

> 実施日時: 2025-01-06
> 依存タスク: 06-repository, 07-use-case

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/repository/pg_unit_of_work.rs` | 新規 | UnitOfWork の PostgreSQL 実装 |
| `backend/src/repository.rs` | 修正 | pg_unit_of_work モジュール追加、re-export |
| `backend/src/presentation.rs` | 新規 | presentation モジュール宣言 |
| `backend/src/presentation/graphql.rs` | 新規 | graphql サブモジュール |
| `backend/src/presentation/graphql/context.rs` | 新規 | UnitOfWork 構築ヘルパー |
| `backend/src/presentation/graphql/types.rs` | 新規 | types サブモジュール |
| `backend/src/presentation/graphql/types/project.rs` | 新規 | Project GraphQL型 |
| `backend/src/presentation/graphql/query.rs` | 新規 | query サブモジュール |
| `backend/src/presentation/graphql/query/project.rs` | 新規 | Project クエリリゾルバー |
| `backend/src/presentation/graphql/schema.rs` | 新規 | スキーマ組み立て |
| `backend/src/main.rs` | 修正 | presentation モジュール追加 |

## ビルド・テスト結果

### cargo check

```
warning: unused imports: `AppSchema` and `build_schema` (src/presentation.rs)
warning: function `create_unit_of_work` is never used (src/presentation/graphql/context.rs)
warning: struct `ProjectQuery` is never constructed (src/presentation/graphql/query/project.rs)
warning: struct `QueryRoot` is never constructed (src/presentation/graphql/schema.rs)
warning: type alias `AppSchema` is never used (src/presentation/graphql/schema.rs)
warning: function `build_schema` is never used (src/presentation/graphql/schema.rs)
warning: struct `Project` is never constructed (src/presentation/graphql/types/project.rs)
warning: struct `PgUnitOfWork` is never constructed (src/repository/pg_unit_of_work.rs)
... (他の層の未使用警告は前タスクから継続)

Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### cargo test

```
running 10 tests
test domain::models::project::tests::test_project_id_new_generates_unique_ids ... ok
test domain::models::project::tests::test_project_new_creates_with_auto_id ... ok
test use_case::project::get_project::tests::test_get_project_not_found ... ok
test use_case::project::get_project::tests::test_get_project_returns_specified_project_from_multiple ... ok
test use_case::project::list_projects::tests::test_list_projects_empty ... ok
test use_case::project::list_projects::tests::test_list_projects_returns_sorted_by_name_asc ... ok
test repository::project_repo::tests::test_find_by_id_returns_none_when_not_exists ... ok
test repository::project_repo::tests::test_find_by_id_returns_project_when_exists ... ok
test repository::project_repo::tests::test_find_all_with_name_asc ... ok
test repository::project_repo::tests::test_find_all_with_created_at_desc ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 設計上の議論と決定

### context.rs の導入

**議論**:
- リゾルバーが `PgPool` と `PgUnitOfWork` を直接参照していることへの懸念
- query レイヤーは頻繁に修正されるため、repository への依存を隔離したい

**検討した選択肢**:

1. **presentation/graphql/context.rs（採用）**
   - GraphQL 固有のコンテキストヘルパーとして配置
   - presentation 層内で完結
   - リゾルバーは `create_unit_of_work(ctx)?` のみで UnitOfWork を取得可能

2. **schema.rs にヘルパー関数を追加（不採用）**
   - ファイル数を増やさない
   - schema.rs の責務が曖昧になる

3. **repository 層に Factory を追加（不採用）**
   - async_graphql の Context を repository 層が知ることになる
   - 依存の方向が逆転

**決定理由**:
- リゾルバーから repository 層への直接依存を排除
- presentation 層内で依存性注入の詳細を隠蔽
- 将来的にトランザクション管理を追加する際の拡張点として機能

## 先送り事項

- [ ] 未使用警告（09-integration で main.rs に統合することで解消予定）
  - `build_schema`, `AppSchema`, `PgUnitOfWork` など
- [ ] エラーハンドリングの改善（現在は `format!("{:?}", e)` で文字列化）

## 次タスクへの申し送り

- `build_schema(pool: PgPool)` でスキーマを構築可能
- `AppSchema` 型を Axum のルーターに登録して使用
- async-graphql-axum の `GraphQL` ハンドラーを使用してルーティング
- スキーマには `PgPool` がコンテキストとして設定済み

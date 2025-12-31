# Testing

テスト実装に関するルール。

## モックオブジェクトの配置

複数のテストファイルで繰り返し利用されるモックオブジェクトは、専用のファイルを作成して再利用する。

```
backend/src/use_case/
├── project/
│   ├── get_project.rs      # テスト内で use_case::test::MockUnitOfWork を使用
│   └── list_projects.rs
└── test/
    └── mock_unit_of_work.rs  # 共通モック
```

**モジュール定義:**

```rust
// src/use_case.rs
pub mod project;

#[cfg(test)]
pub mod test;

// src/use_case/test.rs
pub mod mock_unit_of_work;

pub use mock_unit_of_work::MockUnitOfWork;
```

**使用例:**

```rust
// src/use_case/project/get_project.rs
#[cfg(test)]
mod tests {
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_get_project() {
        let uow = MockUnitOfWork::with_projects(vec![...]);
        // ...
    }
}
```

## アンチパターン

```rust
// ❌ 各テストファイルで同じモックを重複定義
// get_project.rs
struct MockProjectRepository { ... }

// list_projects.rs
struct MockProjectRepository { ... }  // 重複

// ✅ 共通モジュールに集約して再利用
use crate::use_case::test::MockUnitOfWork;
```

## GraphQL 統合テスト

GET 系（クエリ）のテストでは、完全なレスポンス JSON で検証する。

**ファイル配置:**

```
backend/tests/
├── graphql.rs              # テストエントリポイント
├── graphql/
│   ├── schema.rs           # execute_graphql ヘルパー
│   └── projects/
│       ├── project.rs
│       └── projects.rs
└── fixtures/
    └── projects.sql        # テストデータ
```

**テストの書き方:**

```rust
use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_project(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ project(id: "11111111-1111-1111-1111-111111111111") { id name } }"#,
    )
    .await;

    // ✅ 完全な JSON で検証
    assert_eq!(
        data,
        json!({
            "project": {
                "id": "11111111-1111-1111-1111-111111111111",
                "name": "Test Project 1"
            }
        })
    );
}
```

**ポイント:**

- `execute_graphql` を使用してクエリを実行（スキーマ構築とエラーチェックを共通化）
- `assert_eq!` + `json!` マクロで完全なレスポンスを検証
- フィクスチャは `tests/fixtures/` に配置し、相対パスで参照

## チェックリスト

- [ ] 複数箇所で使うモックは `test/` ディレクトリに集約
- [ ] テスト用モジュールは `#[cfg(test)]` で囲む
- [ ] モックは必要最小限の実装のみ
- [ ] GraphQL テストは `execute_graphql` ヘルパーを使用
- [ ] GET 系テストは完全な JSON で検証（部分一致ではなく `assert_eq!`）

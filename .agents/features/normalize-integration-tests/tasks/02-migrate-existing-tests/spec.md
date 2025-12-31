# Task: 既存テストの移行

> Feature: [normalize-integration-tests](../../spec.md)
> 依存: [01-setup-sqlx-test](../01-setup-sqlx-test/)

## 目的

既存の GraphQL テストを新しいテスト基盤（`sqlx::test` + スキーマ直接実行）に移行し、旧ヘルパーを削除する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/tests/graphql/projects/project.rs` | 修正 | 新基盤に移行 |
| `backend/tests/graphql/projects/projects.rs` | 修正 | 新基盤に移行 |
| `backend/tests/graphql/request.rs` | 削除 | 不要になったヘルパー |
| `backend/tests/graphql/test_data/` | 削除 | フィクスチャに置き換え |
| `backend/tests/graphql.rs` | 修正 | モジュール構成の更新 |

---

## 設計詳細

### 1. テストファイルの移行

各テストファイルで以下の変更を行う:

**Before（現在の実装）:**

```rust
use serial_test::serial;
use crate::graphql::request::{assert_no_errors, execute_graphql, get_data};
use crate::graphql::test_data::project::{delete_test_project, insert_test_project};

#[tokio::test]
#[serial]
async fn test_returns_project() {
    let project = insert_test_project("test_project").await;
    let json = execute_graphql(&format!(r#"{{ project(id: "{}") {{ id }} }}"#, project.id.0)).await;
    assert_no_errors(&json);
    delete_test_project(&project).await;
}
```

**After（新しい実装）:**

```rust
use async_graphql::Request;
use sqlx::PgPool;
use crate::graphql::schema::create_test_schema;

#[sqlx::test(migrations = "./migrations", fixtures("projects"))]
async fn test_returns_project(pool: PgPool) {
    let schema = create_test_schema(pool);

    let response = schema
        .execute(r#"{ project(id: "11111111-1111-1111-1111-111111111111") { id name } }"#)
        .await;

    assert!(response.errors.is_empty(), "GraphQL errors: {:?}", response.errors);

    let data = response.data.into_json().unwrap();
    assert_eq!(data["project"]["id"], "11111111-1111-1111-1111-111111111111");
}
```

### 2. 移行対象テスト一覧

| テスト | フィクスチャ | 備考 |
|--------|--------------|------|
| `project.rs::test_returns_null_when_not_found` | なし | 存在しない ID のテスト |
| `project.rs::test_returns_project` | `projects` | ID 指定でプロジェクト取得 |
| `projects.rs::test_returns_list` | なし | 空でも動作確認 |
| `projects.rs::test_contains_inserted_project` | `projects` | リストにフィクスチャデータが含まれる |

### 3. 削除するファイル

旧テスト基盤の以下のファイルを削除:

- `tests/graphql/request.rs` - HTTP ベースのヘルパー
- `tests/graphql/test_data.rs` - モジュール定義
- `tests/graphql/test_data/project.rs` - 手動データ操作ヘルパー

### 4. モジュール構成の更新

`tests/graphql.rs` を更新:

```rust
mod graphql {
    pub mod projects;
    pub mod schema;  // 新規追加
    // request, test_data は削除
}
```

### 5. アサーションヘルパー（任意）

繰り返し使用するアサーションがあれば `tests/graphql/assert.rs` 等に集約可能。
ただし、現時点では最小限の実装で進め、必要に応じて追加する。

---

## 完了条件

- [ ] 全テストが `sqlx::test` + スキーマ直接実行に移行されている
- [ ] `serial_test` の使用がなくなっている
- [ ] 旧ヘルパー（`request.rs`, `test_data/`）が削除されている
- [ ] `docker compose exec backend bash -c "cargo test"` が成功する
- [ ] テストが並列実行される（`--test-threads=1` なしで動作）

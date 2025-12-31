# Task: テスト基盤のセットアップ

> Feature: [normalize-integration-tests](../../spec.md)
> 依存: なし

## 目的

`sqlx::test` と `async-graphql` のスキーマ直接実行を使用した新しいテスト基盤を構築する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/Cargo.toml` | 修正 | sqlx の test-util feature 追加、serial_test 削除 |
| `backend/tests/graphql/schema.rs` | 新規 | テスト用スキーマビルダー |
| `backend/tests/fixtures/` | 新規 | SQL フィクスチャディレクトリ |

---

## 設計詳細

### 1. Cargo.toml の変更

`[dev-dependencies]` セクションを更新:

- **追加**: sqlx に `test-util` feature を追加（`#[sqlx::test]` マクロを使用するため）
- **削除**: `serial_test` クレート（`sqlx::test` による分離で不要に）
- **削除**: `tower`, `http-body-util`（HTTP レイヤーテストが不要に）

```toml
[dev-dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "test-util"] }
tokio = { version = "1", features = ["test-util", "macros"] }
```

### 2. テスト用スキーマビルダー

`tests/graphql/schema.rs` に、テスト用の `AppSchema` を構築するヘルパーを実装:

- 引数として `PgPool` を受け取る
- `sqlx::test` から渡される pool を使用してスキーマを構築
- 既存の `build_schema()` を再利用

```rust
// tests/graphql/schema.rs
use bake_loose::presentation::graphql::build_schema;
use sqlx::PgPool;

pub fn create_test_schema(pool: PgPool) -> bake_loose::presentation::graphql::AppSchema {
    build_schema(pool)
}
```

### 3. フィクスチャディレクトリ

`tests/fixtures/` にテストデータ用 SQL を配置:

- `sqlx::test` の `fixtures(...)` 属性で参照
- 各エンティティごとにファイルを分離
- INSERT 文で必要最小限のテストデータを定義

ディレクトリ構造:

```
backend/tests/fixtures/
├── projects.sql      # Project のテストデータ
└── ...               # 将来的に Trial, Feedback 等を追加
```

フィクスチャ例 (`projects.sql`):

```sql
-- テスト用プロジェクト
INSERT INTO projects (id, name, created_at, updated_at)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'Test Project 1', NOW(), NOW()),
    ('22222222-2222-2222-2222-222222222222', 'Test Project 2', NOW(), NOW());
```

### 4. テストの書き方（参考）

新しいテスト基盤での記述方法:

```rust
use async_graphql::Request;
use sqlx::PgPool;

mod schema;

#[sqlx::test(migrations = "./migrations", fixtures("projects"))]
async fn test_returns_project(pool: PgPool) {
    let schema = schema::create_test_schema(pool);

    let response = schema
        .execute(r#"{ project(id: "11111111-1111-1111-1111-111111111111") { id name } }"#)
        .await;

    assert!(response.errors.is_empty());
    // ...
}
```

---

## 完了条件

- [ ] Cargo.toml の dev-dependencies が更新されている
- [ ] `tests/graphql/schema.rs` が作成されている
- [ ] `tests/fixtures/` ディレクトリと初期フィクスチャが作成されている
- [ ] `docker compose exec backend bash -c "cargo check"` が成功する

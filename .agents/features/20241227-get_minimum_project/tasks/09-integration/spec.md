# Task: main.rs の統合・Axum ルーティング

> Feature: [get-project](../../spec.md)
> 依存: 08-presentation

## 目的
すべてのレイヤーを統合し、GraphQL エンドポイントを公開する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/main.rs` | 修正 | GraphQL ルーティングの追加、DB接続 |
| `backend/.env.example` | 新規 | 環境変数のサンプル |

---

## 設計詳細

### main.rs の構成

1. **環境変数の読み込み**
   - `dotenvy::dotenv()` で .env ファイルを読み込み
   - `DATABASE_URL` を取得

2. **ロギングの初期化**
   - `tracing_subscriber` で初期化

3. **DB接続プールの作成**
   - `infrastructure::database::create_pool()` を呼び出し

4. **GraphQL スキーマの構築**
   - `presentation::graphql::build_schema(pool)` を呼び出し

5. **Axum ルーティング**
   - `GET /health`: ヘルスチェック（既存）
   - `POST /graphql`: GraphQL エンドポイント
   - `GET /graphql`: GraphQL Playground（開発用）

### エンドポイント

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/` | ヘルスチェック |
| GET | `/health` | ヘルスチェック |
| POST | `/graphql` | GraphQL クエリ/ミューテーション |
| GET | `/graphql` | GraphQL Playground |

### 環境変数

| 変数 | 説明 | 例 |
|------|------|-----|
| `DATABASE_URL` | PostgreSQL 接続URL | `postgres://postgres:password@localhost:5432/bake_loose` |

### GraphQL Playground

開発時の動作確認用に GraphQL Playground を有効化する。
`async_graphql_axum` の `GraphQLPlayground` を使用。

---

## 動作確認

### GraphQL クエリ例

```graphql
# すべてのプロジェクトを取得
query {
  projects {
    id
    name
  }
}

# IDでプロジェクトを取得
query {
  project(id: "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11") {
    id
    name
  }
}
```

### curl での確認

```bash
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ projects { id name } }"}'
```

---

## 完了条件

- [ ] DB接続が初期化されている
- [ ] GraphQL エンドポイントが `/graphql` で公開されている
- [ ] GraphQL Playground が開発環境で利用可能
- [ ] 既存のヘルスチェックエンドポイントが動作する
- [ ] `cargo run` でサーバーが起動する
- [ ] GraphQL クエリでプロジェクトが取得できる

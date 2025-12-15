# Infrastructure Layer Skill

## 概要

Infrastructure層はアプリケーションの技術的基盤を提供する層であり、データベース接続、環境変数の読み込みなど純粋な技術機能を担当する。

---

## 基本原則

### 技術基盤としての役割

Infrastructure層は **アプリケーションが動作するための技術的な土台** を提供する。

```
┌─────────────────────────────────────────────────────────────┐
│  repository層                                               │
│    ↓ 利用                                                   │
│  infrastructure層（DB接続プール、設定管理など）                │
│    ↓ 接続                                                   │
│  外部リソース（PostgreSQL、Redis、外部API など）              │
└─────────────────────────────────────────────────────────────┘
```

### 依存の方向

- **Infrastructure層はドメイン層に依存しない**
- ビジネスロジックやシステム要件への依存禁止
- 外部クレート（sqlx, redis 等）への依存は許可される
- repository層から利用される

### 責務の範囲

| やること | やらないこと |
|---------|-------------|
| DB接続プールの作成・管理 | ビジネスロジック |
| 環境変数からの設定読み込み | バリデーション（ビジネス的な） |
| 外部サービスへの接続管理 | ドメインモデルの操作 |
| 接続のヘルスチェック | SQL の発行（repository層の責務） |
| 接続設定の抽象化 | トランザクション管理（use_case層の責務） |

**重要:** Infrastructure層はドメイン層のモデルやアクションを知らない。純粋な技術的機能のみを提供する。

---

## ファイル配置

```
backend/src/infrastructure/
├── database.rs          # データベース接続プール管理
├── config.rs            # 環境変数・設定管理
└── health.rs            # ヘルスチェック機能（オプション）
```

**命名規則:**
- 技術機能ごとに1ファイル
- 具体的で明確な名前を使用

### 1ファイル1関心事の原則

各ファイルは **単一の技術的関心事** のみを扱う：

- `database.rs`: DB接続プールの作成・管理のみ
- `config.rs`: 環境変数の読み込みのみ
- `health.rs`: ヘルスチェックのみ

**関心事間の依存は外部で解決する：**

```rust
// ❌ database.rs 内で環境変数を直接読み込む
pub async fn create_pool() -> Result<PgPool, Error> {
    let url = std::env::var("DATABASE_URL")?;  // NG: 環境変数の関心事が混入
    connect(&url).await
}

// ✅ 接続情報は引数として受け取る
pub async fn create_pool(url: &str) -> Result<PgPool, Error> {
    connect(url).await
}
// 環境変数の取得 → DB接続への受け渡しは infrastructure の外側（main.rs等）で行う
```

これにより：
- 各ファイルの責務が明確になる
- テスト時に環境変数に依存せず接続情報を注入できる
- 設定の取得元を変更しても DB 接続コードに影響しない

---

## 記載すべきコードの種類

Infrastructure層では以下の種類のコードを記載する：

### 1. データベース接続

- 接続プールの作成関数
- 接続設定の構造体（URL、最大接続数、タイムアウト等）
- 接続確認用の関数

### 2. 設定管理

- アプリケーション設定の構造体
- 環境変数からの読み込み関数
- 設定エラー型の定義

### 3. ヘルスチェック（オプション）

- 各コンポーネント（DB等）の健全性確認
- ヘルスステータスの構造体

### 4. 外部サービス連携（将来的な拡張）

- 外部サービスクライアントの作成関数
- 接続設定の構造体
- Redis, 外部API ...

---

## アンチパターン

### NG: ビジネスロジックの実装

```rust
// ❌ infrastructure層でビジネスロジックを実装
pub async fn create_pool_and_seed(url: &str) -> Result<PgPool, Error> {
    let pool = create_pool(url).await?;
    // 初期データ投入（NG: ビジネスロジック）
    let default_project = Project::new("デフォルト".to_string(), None, None);
    // ...
}

// ✅ 純粋な技術機能のみを提供
pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    // 接続プール作成のみ
}
```

### NG: ドメインモデルへの依存

```rust
// ❌ infrastructure層でドメインモデルをインポート
use crate::domain::models::project::Project;

// ✅ ドメインモデルに依存しない
// infrastructure層はドメイン層の存在を知らない
```

### NG: SQL の発行

```rust
// ❌ infrastructure層でエンティティ操作のSQLを発行
pub async fn get_all_projects(pool: &PgPool) -> Result<Vec<ProjectRow>, Error> {
    sqlx::query_as("SELECT * FROM projects").fetch_all(pool).await
}

// ✅ SQL発行は repository層の責務
// infrastructure層は接続プールを提供するのみ
```

### NG: repository層との混同

```rust
// ❌ infrastructure層にリポジトリ実装を置く
// backend/src/infrastructure/project_repo.rs  ← NG

// ✅ リポジトリ実装は repository層に配置
// backend/src/repository/project_repo.rs      ← OK
```

### NG: 設定のハードコーディング

```rust
// ❌ 接続設定をハードコーディング
pub async fn create_pool() -> Result<PgPool, Error> {
    connect("postgres://localhost:5432/mydb").await  // ハードコーディング（NG）
}

// ✅ 接続URLは引数として受け取る（プレーンな文字列）
pub async fn create_pool(url: &str) -> Result<PgPool, Error> {
    connect(url).await
}
// 呼び出し側（main.rs等）で環境変数から取得した値を渡す
```

---

## チェックリスト

Infrastructure層のコードをレビューする際は以下を確認：

### 基本原則
- [ ] ビジネスロジックを含んでいない
- [ ] ドメイン層への依存がない
- [ ] SQL 発行（エンティティ操作）を行っていない
- [ ] 設定がハードコーディングされていない

### データベース接続
- [ ] 接続プールの設定が外部から注入可能
- [ ] タイムアウト設定が適切
- [ ] 接続数の上限が設定されている

### 設定管理
- [ ] 環境変数から設定を読み込んでいる
- [ ] 必須の環境変数が明確になっている
- [ ] ConfigError が適切に定義されている

### ファイル配置
- [ ] `src/infrastructure/` 配下に配置されている
- [ ] 技術機能ごとにファイルが分かれている
- [ ] repository 実装を含んでいない

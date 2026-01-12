---
paths: backend/src/infrastructure/**/*.rs
---

# Infrastructure Layer

Infrastructure層はアプリケーションの技術的基盤を提供。DB接続、設定管理など純粋な技術機能を担当。

## 基本原則

- **依存禁止**: プロジェクト内の他層（domain, use_case, ports, repository, presentation）
- **許可される依存**: 外部クレート（sqlx, redis等）のみ
- **責務**: 技術的な土台の提供のみ

**やること**:
- DB接続プール管理
- 環境変数からの設定読み込み
- ...

**やらないこと**:
- ビジネスロジック
- ドメインモデルの操作
- SQL発行（repository層の責務）

## ファイル配置

```
backend/src/infrastructure/
├── database.rs    # DB接続プール管理
├── config.rs      # 環境変数・設定管理
└── ...
```

**1ファイル1関心事**: 各ファイルは単一の技術的関心事のみを扱う。

## 実装例

```rust
// src/infrastructure/database.rs

pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
}
```

```rust
// src/infrastructure/config.rs

pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            server_port: std::env::var("PORT")?.parse()?,
        })
    }
}
```

## アンチパターン

```rust
// ❌ ドメインモデルへの依存
use crate::domain::models::project::Project;

// ❌ ビジネスロジック
let default_project = Project::new("デフォルト".to_string(), None, None);

// ❌ エンティティ操作のSQL発行
pub async fn get_all_projects(pool: &PgPool) -> ... { ... }

// ❌ 設定のハードコーディング
connect("postgres://localhost:5432/mydb").await

// ✅ 純粋な技術機能のみ、設定は引数で受け取る
```

## チェックリスト

- [ ] プロジェクト内の他層への依存がない
- [ ] ビジネスロジックを含まない
- [ ] SQL発行（エンティティ操作）を行っていない
- [ ] 設定がハードコーディングされていない
- [ ] 接続情報は外部から注入可能

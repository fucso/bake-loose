# Domain Layer

ドメイン層はアプリケーションの最内層であり、純粋なビジネスロジックを担当する。

## 基本原則

- **依存禁止**: 外部クレート（sqlx, axum 等）、I/O操作、永続化の詳細を知らない
- **許可される依存**: Rust標準ライブラリ、serde（シリアライズのみ）
- **純粋関数**: 副作用を持たない純粋関数で構成

## ファイル配置

```
backend/src/domain/
├── models/          # Project, Trial, ...
└── actions/         # 1アクション1ファイル
    ├── project/
    ├── trial/
    └── ...
```

## モデル定義

```rust
// src/domain/models/project.rs

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    id: ProjectId,
    name: String,
    // ...
}

// ID は NewType パターン
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);
```

**モデルのメソッド**: ファクトリ（`new()`）とゲッターのみ。ロジックはアクションに集約。

## アクション定義

validate / execute 分離パターンを採用:

```rust
// src/domain/actions/project/update_project_name.rs

pub struct Command {
    pub new_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyName,
    NameTooLong { max: usize, actual: usize },
    CannotUpdateArchived,
}

/// バリデーション
pub fn validate(state: &Project, command: &Command) -> Result<(), Error> {
    if command.new_name.is_empty() { return Err(Error::EmptyName); }
    if state.status() == ProjectStatus::Archived { return Err(Error::CannotUpdateArchived); }
    Ok(())
}

/// 状態遷移（validate成功前提）
pub fn execute(state: Project, command: Command) -> Project {
    Project { name: command.new_name, ..state }
}

/// validate + execute
pub fn run(state: Project, command: Command) -> Result<Project, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}
```

## アンチパターン

```rust
// ❌ 外部依存
use sqlx::PgPool;
pub async fn run(pool: &PgPool, ...) { ... }

// ❌ 可変参照による状態変更
pub fn update_name(&mut self, name: String) { self.name = name; }

// ✅ 必要なリソースは引数で受け取り、新しいインスタンスを返す
pub fn run(state: Project, command: Command) -> Result<Project, Error> { ... }
```

## チェックリスト

- [ ] 外部クレートへの依存がない
- [ ] I/O操作を行っていない
- [ ] ID は NewType パターン
- [ ] 1アクション1ファイル
- [ ] validate / execute / run が分離されている
- [ ] エラー型は種類のみ（メッセージを含まない）

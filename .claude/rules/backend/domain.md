---
paths: backend/src/domain/**/*.rs
---

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

**モデルのメソッド**:
- **ファクトリ**: `new()` - 新規作成時に使用
- **ゲッター**: フィールドへのアクセス
- **ミューテーションメソッド**: `set_xxx()`, `add_xxx()`, `remove_xxx()`, `complete()` など - 状態変更を担当
  - 内部で `updated_at` を自動更新
  - バリデーションは含まない（アクションで事前に検証）

**`from_raw()` メソッド**: リポジトリ層でのDB再構築専用。Action層やUseCase層では使用禁止。

```rust
// ✅ Repository層: from_raw() で DB から再構築
impl TrialRepository for PgTrialRepository {
    async fn find(&self, id: &TrialId) -> Option<Trial> {
        Trial::from_raw(row.id, row.name, ...)  // OK
    }
}

// ✅ Action層: ミューテーションメソッドで状態変更
pub fn execute(mut state: Trial, command: Command) -> Trial {
    state.set_name(Some(command.new_name));  // OK
    state
}

// ❌ Action層で from_raw() は使用禁止
pub fn execute(state: Trial, command: Command) -> Trial {
    Trial::from_raw(state.id(), command.new_name, ...)  // NG
}
```

**親子関係のある集約の構築順序**: 必ず親→子→孫の順で構築する。

```rust
// ✅ 正しい順序: Trial (親) → Step (子) → Parameter (孫)
let trial = Trial::new(project_id, name, memo);
let step = Step::new(trial.id().clone(), step_name, position);
let parameter = Parameter::new(step.id().clone(), content);
step.add_parameter(parameter);
trial.add_step(step);

// ❌ 誤った順序: 子を先に作成
let step = Step::new(trial_id, ...);  // trial_id がまだ存在しない
let trial = Trial::new(...);
```

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

// ❌ Action層で from_raw() を使用
pub fn execute(state: Trial, command: Command) -> Trial {
    Trial::from_raw(state.id(), command.new_name, ...)  // from_raw はリポジトリ層専用
}

// ❌ ミューテーションメソッド内でバリデーション
pub fn set_name(&mut self, name: String) {
    if name.is_empty() { panic!("..."); }  // バリデーションはActionで行う
    self.name = name;
}

// ✅ Action層: ミューテーションメソッドで状態変更
pub fn execute(mut state: Project, command: Command) -> Project {
    state.set_name(command.new_name);
    state
}
```

## チェックリスト

- [ ] 外部クレートへの依存がない
- [ ] I/O操作を行っていない
- [ ] ID は NewType パターン
- [ ] 1アクション1ファイル
- [ ] validate / execute / run が分離されている
- [ ] エラー型は種類のみ（メッセージを含まない）
- [ ] `from_raw()` をAction層で使用していない（リポジトリ層専用）
- [ ] 親子関係のある集約は親→子→孫の順で構築
- [ ] 状態変更はミューテーションメソッド経由

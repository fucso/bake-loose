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
├── actions/         # 1アクション1ファイル
│   ├── project/
│   ├── trial/
│   └── ...
└── validators/      # ドメインオブジェクト別のバリデーター
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

## バリデーター定義

ドメイン的に意味のあるバリデーションロジックは `validators/` に切り出す。ドメインオブジェクト単位でサブディレクトリを作り、条件の種類ごとにファイルを分ける。

**切り出す基準**: ドメインモデルの状態に関わるチェック（ステータス、存在確認など）が対象。入力値の形式チェック（文字数、空文字など）はアクション固有の関心事であり、バリデーターとして切り出さないこともある。

**設計思想**:
- バリデーターは「条件チェックのロジックとそのエラー型」を所有する
- アクションの Error enum は「このアクションが返しうるエラーの集約」であり、アクションの責務
- 外部（use case、presentation）はアクションの Error のみを意識し、バリデーターの存在を知らない

```rust
// src/domain/validators/trial/trial_status_validator.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
}

pub fn require_in_progress(trial: &Trial) -> Result<(), Error> {
    if trial.status() == &TrialStatus::Completed {
        return Err(Error::TrialAlreadyCompleted);
    }
    Ok(())
}
```

**アクションからの使用パターン**:

バリデーターが1つだけの場合、アクションの Error を `pub use` で再エクスポートする:

```rust
// ✅ 単一バリデーター: pub use で Error を再エクスポート
pub use trial_status_validator::Error;

pub fn validate(state: &Trial) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    Ok(())
}
```

複数バリデーターを組み合わせる場合、アクションが独自の Error enum を定義して集約し、`.map_err` でマッピングする:

```rust
// ✅ 複数バリデーター: アクション固有の Error に集約
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
}

pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)
        .map_err(|_| Error::TrialAlreadyCompleted)?;
    step_existence_validator::require_exists(state, &command.step_id)
        .map_err(|_| Error::StepNotFound)?;
    let step = state
        .steps()
        .iter()
        .find(|step| step.id() == &command.step_id)
        .expect("step existence already validated");
    step_status_validator::require_in_progress(step)
        .map_err(|_| Error::StepAlreadyCompleted)?;
    Ok(())
}
```

**Validator が実態（モデル参照）を返さない**: Validator は判定結果とエラーのみを返し（`Result<(), Error>`）、
モデルの実態が必要な場合はアクションの `validate()` 内で `state` から取得する。
Validator が実態まで返すと、Validator の責務が「条件チェック」を超えて「データ取得」まで広がってしまう。

アクション Error のネストした型（`InvalidParameter` の reason 等）も同様に `pub use` で参照する:

```rust
// ✅ ネストした Error 型も pub use で参照
pub use parameter_validator::Error as ParameterValidationError;

pub enum Error {
    InvalidParameter { parameter_index: usize, reason: ParameterValidationError },
}
```

**テスト方針**:
- バリデーター側: 条件ごとの詳細なテスト（境界値、各パターン）
- アクション側: バリデーションが適用されることの確認（最低限）＋アクション固有の分岐

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

## アクションの責務範囲

**単一責務の原則**: 各アクションは1つの集約ルートまたは密接に関連するエンティティのみを操作する。

**集約境界の判断基準**:
- Trial と Step の関係: 独立したライフサイクルを持つ → **別アクションで管理**
- Step と Parameter の関係: Parameter は Step なしに存在できない → **同一アクションで管理可能**

```rust
// ✅ create_trial: Trial のみ作成
pub fn execute(command: Command) -> Trial {
    Trial::new(command.project_id, command.name, command.memo)
}

// ✅ add_step: Step + Parameter を作成（密接な関係）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let step = Step::new(...);
    for param in command.parameters {
        step.add_parameter(Parameter::new(...));
    }
    state.add_step(step);
    state
}

// ❌ create_trial が Step + Parameter も作成（責務過多）
pub fn execute(command: Command) -> Trial {
    let trial = Trial::new(...);
    for step_input in command.steps {
        // Step 作成ロジックが add_step と重複する
    }
    trial
}
```

**use case でのオーケストレーション**: 複数の集約操作が必要な場合は use case が複数アクションを順次呼び出す。

```rust
// use_case/trial/create_trial.rs
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial 作成（単一アクション）
    let mut trial = create_trial::run(command)?;

    // 2. Step 追加（別アクションをループ呼び出し）
    for step_command in input.steps {
        trial = add_step::run(trial, step_command)?;
    }

    // 3. 永続化
    uow.trial_repository().save(&trial).await?;
    Ok(trial)
}
```

**型の再利用**: アクションの入力型を use case で直接使用し、重複定義を避ける。

```rust
// ✅ use case が domain action の型を直接使用
pub struct Input {
    pub steps: Vec<add_step::Command>,  // domain action の型を再利用
}

// ❌ use case で同じ構造の型を重複定義
pub struct StepInput { ... }  // add_step::Command と同じ構造
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
- [ ] アクションは単一責務（密接な関係のエンティティのみ同時操作）
- [ ] 他のアクションと同じロジックを重複実装していない
- [ ] ドメインモデルの状態に関わる共通バリデーションは `validators/` に切り出している
- [ ] バリデーターの Error 型をアクション外部に直接露出していない（アクションの Error に集約）

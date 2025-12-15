# Domain Layer Skill

## 概要

ドメイン層はアプリケーションの最内層であり、純粋なビジネスロジックを担当する。
このスキルはドメイン層（モデル・アクション）の設計・実装ルールを定義する。

---

## 基本原則

### 依存の方向

- **ドメイン層は何にも依存しない**（最内層）
- 外部クレート（データベース、Web フレームワーク等）への依存禁止
- 許可される依存: Rust 標準ライブラリ、serde（シリアライズのみ）

### 純粋性

- ドメインのコードは副作用を持たない純粋関数で構成する
- I/O 操作（DB アクセス、ファイル操作、HTTP 通信）は一切行わないし、それらがどのように行われるかにも関与しない
- 永続化の詳細を知らない

---

## モデルとアクション：すべての起点

> **このプロジェクトにおいて、ドメイン層のモデルとアクションはすべての設計・実装の起点である。**

### なぜモデルとアクションが最重要なのか

アプリケーションの本質的な価値は「何ができるか」（機能）ではなく「何を表現しているか」（ドメイン）にある。
データベーススキーマでもAPIエンドポイントでもなく、**ビジネスの概念（モデル）とその振る舞い（アクション）が設計の出発点**となる。

```
モデル・アクションを定義
    ↓
ports層（永続化の境界）を定義
    ↓
ユースケース層（フローのオーケストレーション）を実装
    ↓
リポジトリ層（永続化の実装）を実装
    ↓
プレゼンテーション層（外部インターフェース）を実装
```

### モデルとは

**モデル**はビジネスの概念を表現するデータ構造。
このプロジェクトでは以下が主要なモデルとなる：

| モデル | 役割 |
|--------|------|
| **Project** | 調理テーマ（例：「ピザ生地研究」「カンパーニュ」） |
| **Trial** | 特定のProjectに対する1回の試行記録 |
| **Feedback** | 試行に対する評価・コメント |

### アクションとは

**アクション**はモデルに対するビジネス上の操作を表す純粋関数。
「プロジェクトを作成する」「試行を記録する」「フィードバックを追加する」といった操作がアクションとなる。

| 対象 | 主なアクション |
|------|---------------|
| **Project** | `create_project`, `update_project`, `archive_project` |
| **Trial** | `record_trial`, `update_trial_parameters`, `delete_trial` |
| **Feedback** | `add_feedback`, `update_feedback`, `delete_feedback` |

### 設計時の思考順序

新機能を設計する際は、必ず以下の順序で考える：

1. **どのモデルが関係するか？** - 既存モデルの変更か、新規モデルの追加か
2. **どのアクションが必要か？** - モデルに対してどのような操作が必要か
3. **バリデーションとビジネスルールは？** - アクションの成功/失敗条件は何か
4. **永続化はどうするか？** - ports層以降で考える（ここではまだ考えない）

---

## ファイル配置

```
backend/src/domain/
├── models/
│   ├── project.rs
│   ├── trial.rs
│   └── feedback.rs
└── actions/
    ├── project/
    │   ├── create_project.rs
    │   ├── update_project_name.rs
    │   └── archive_project.rs
    ├── trial/
    │   ├── record_trial.rs
    │   └── update_trial_parameters.rs
    └── feedback/
        ├── add_feedback.rs
        └── update_feedback.rs
```

**1アクション1ファイル** の原則を採用する。これにより：
- 責務が明確になる
- 変更影響範囲が限定される
- コードレビューがしやすくなる

---

## モデル定義ルール

### 構造体の定義

```rust
use serde::{Deserialize, Serialize};

/// プロジェクト（調理テーマ）を表すドメインモデル
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    id: ProjectId,
    name: String,
    description: Option<String>,
    goal: Option<String>,
    status: ProjectStatus,
    trial_ids: Vec<TrialId>,
}
```

### ID 型の定義

型安全性のため、各エンティティの ID は NewType パターンで定義する:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### 列挙型（状態）の定義

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        Self::Active
    }
}
```

### モデルのメソッド

モデルには **データの保持とカプセル化** のみを担当させる。
ビジネスロジックはアクションが担当するため、モデルには以下のメソッドのみを定義する：

| 種類 | 役割 | 例 |
|------|------|-----|
| ファクトリメソッド | 新規インスタンス生成 | `new()` |
| ゲッター | フィールドへの読み取りアクセス | `id()`, `name()`, `status()` |

**モデルにロジックを持たせない理由：**
- ロジックがアクションに集約され、変更箇所が明確になる
- モデルは純粋なデータ構造として保たれ、テストが容易になる
- 1アクション1ファイルの原則と整合する

```rust
impl Project {
    /// 新しいプロジェクトを作成（ファクトリメソッド）
    pub fn new(name: String, description: Option<String>, goal: Option<String>) -> Self {
        Self {
            id: ProjectId::new(),
            name,
            description,
            goal,
            status: ProjectStatus::default(),
            trial_ids: Vec::new(),
        }
    }

    // ゲッター
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> ProjectStatus {
        self.status
    }
}
```

---

## アクション定義ルール

### アクションとは

アクションはドメインモデルの状態を変更するビジネスロジックを表す純粋関数。
**バリデーション（validate）と実行（execute）を分離** するパターンを採用する。

参考: [永続化を切り離したドメインモデリング](https://zenn.dev/jtechjapan_pub/articles/fc9878ec69b6a1)

### validate / execute 分離パターン

| 関数 | 役割 | 戻り値 |
|------|------|--------|
| `validate` | 実行可能かを判定（バリデーション・ビジネスルール） | `Result<(), Error>` |
| `execute` | 状態遷移を実行（validate 成功前提） | 新しいモデル |
| `run` | validate + execute を一括実行 | `Result<Model, Error>` |

### アクションの実装パターン

```rust
// backend/src/domain/actions/project/update_project_name.rs

use crate::domain::models::project::{Project, ProjectStatus};

/// コマンド（入力）
pub struct Command {
    pub new_name: String,
}

/// エラー型（種類のみを定義、メッセージは presentation 層で変換）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyName,
    NameTooLong { max: usize, actual: usize },
    CannotUpdateArchived,
}

/// バリデーション（実行可能かを判定）
/// - 入力値のバリデーション
/// - ビジネスルールのチェック
pub fn validate(state: &Project, command: &Command) -> Result<(), Error> {
    // 入力値バリデーション
    if command.new_name.is_empty() {
        return Err(Error::EmptyName);
    }
    if command.new_name.len() > 100 {
        return Err(Error::NameTooLong {
            max: 100,
            actual: command.new_name.len(),
        });
    }

    // ビジネスルールチェック
    if state.status() == ProjectStatus::Archived {
        return Err(Error::CannotUpdateArchived);
    }

    Ok(())
}

/// 実行（状態遷移）
/// 注意: validate が成功した前提で呼び出すこと
pub fn execute(state: Project, command: Command) -> Project {
    Project {
        name: command.new_name,
        ..state
    }
}

/// validate + execute を一括実行
pub fn run(state: Project, command: Command) -> Result<Project, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}
```

### 各関数の使い分け

| ユースケース | 使う関数 |
|-------------|---------|
| 通常の操作 | `run` を使用 |
| イベントリプレイ | `execute` のみ使用（過去のイベントは既に検証済み） |
| 事前チェック | `validate` のみ使用（実行せずに可否だけ確認） |

### アクションの命名規則

| 操作 | ファイル名 | 例 |
|------|-----------|-----|
| 作成 | `create_*.rs` | `create_project.rs` |
| 更新 | `update_*.rs` | `update_project_name.rs` |
| 削除 | `delete_*.rs` | `delete_trial.rs` |
| 追加 | `add_*.rs` | `add_feedback.rs` |
| アーカイブ | `archive_*.rs` | `archive_project.rs` |

### ファイル内の構造

各アクションファイルは以下の構造を持つ：

```rust
// 1. Command 構造体（入力）
pub struct Command { ... }

// 2. Error 列挙型（種類のみ、メッセージなし）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error { ... }

// 3. validate 関数
pub fn validate(state: &Model, command: &Command) -> Result<(), Error> { ... }

// 4. execute 関数
pub fn execute(state: Model, command: Command) -> Model { ... }

// 5. run 関数
pub fn run(state: Model, command: Command) -> Result<Model, Error> { ... }
```

---

## アンチパターン

### NG: 外部依存（データベース）

```rust
// ❌ アクション内でデータベースにアクセス
use sqlx::PgPool;

pub async fn run(
    pool: &PgPool,
    project_id: ProjectId,
    command: Command,
) -> Result<Project, Error> {
    // アクション内で DB から取得している
    let project = sqlx::query_as("SELECT * FROM projects WHERE id = $1")
        .bind(&project_id)
        .fetch_one(pool)
        .await?;

    validate(&project, &command)?;
    Ok(execute(project, command))
}

// ✅ 必要なリソースは引数で受け取る
pub fn run(state: Project, command: Command) -> Result<Project, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}
// DB からの取得はユースケース層が行い、取得済みの Project を渡す
```

### NG: 外部依存（ファイル操作）

```rust
// ❌ アクション内でファイルを読み込む
pub fn run(config_path: &str, command: Command) -> Result<Project, Error> {
    let config = std::fs::read_to_string(config_path)?;
    let settings: Settings = serde_json::from_str(&config)?;
    // ...
}

// ✅ 必要なリソースは引数で受け取る
pub fn run(state: Project, settings: &Settings, command: Command) -> Result<Project, Error> {
    validate(&state, &settings, &command)?;
    Ok(execute(state, command))
}
// ファイル読み込みはユースケース層が行い、取得済みの Settings を渡す
```

### NG: 可変参照による状態変更

```rust
// ❌ &mut self による状態変更（トレースしにくい）
impl Project {
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
}

// ✅ 新しいインスタンスを返す（execute 関数）
pub fn execute(state: Project, command: Command) -> Project {
    Project { name: command.new_name, ..state }
}
```

### NG: validate と execute を分離しない

```rust
// ❌ バリデーションと実行が混在
pub fn update_name(project: Project, new_name: String) -> Result<Project, Error> {
    if new_name.is_empty() {
        return Err(Error::EmptyName);
    }
    Ok(Project { name: new_name, ..project })
}

// ✅ validate / execute を分離
pub fn validate(state: &Project, command: &Command) -> Result<(), Error> { ... }
pub fn execute(state: Project, command: Command) -> Project { ... }
pub fn run(state: Project, command: Command) -> Result<Project, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}
```

---

## エラー型の集約

複数のアクションエラーを集約する場合は、以下のパターンを使用する：

```rust
/// ドメイン層で発生するエラーの集約
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    Project(project::Error),
    Trial(trial::Error),
    Feedback(feedback::Error),
}

// From トレイトで自動変換
impl From<update_project_name::Error> for DomainError {
    fn from(e: update_project_name::Error) -> Self {
        DomainError::Project(project::Error::UpdateName(e))
    }
}
```

---

## テスト

### ドメイン層のテスト方針

ドメイン層は純粋関数で構成されるため、モックなしで単体テストが可能。
validate / execute / run それぞれをテストする。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // テスト用のヘルパー
    fn create_active_project() -> Project {
        Project::new("テストプロジェクト".to_string(), None, None)
    }

    // validate のテスト
    mod validate_tests {
        use super::*;

        #[test]
        fn 有効な名前なら成功() {
            let project = create_active_project();
            let command = Command { new_name: "新しい名前".to_string() };

            assert!(validate(&project, &command).is_ok());
        }

        #[test]
        fn 空の名前ならエラー() {
            let project = create_active_project();
            let command = Command { new_name: "".to_string() };

            assert!(matches!(
                validate(&project, &command),
                Err(Error::EmptyName)
            ));
        }

        #[test]
        fn アーカイブ済みならエラー() {
            let project = create_archived_project();
            let command = Command { new_name: "新しい名前".to_string() };

            assert!(matches!(
                validate(&project, &command),
                Err(Error::CannotUpdateArchived)
            ));
        }
    }

    // execute のテスト
    mod execute_tests {
        use super::*;

        #[test]
        fn 名前が更新される() {
            let project = create_active_project();
            let command = Command { new_name: "更新後の名前".to_string() };

            let result = execute(project, command);

            assert_eq!(result.name(), "更新後の名前");
        }
    }

    // run のテスト（統合）
    mod run_tests {
        use super::*;

        #[test]
        fn 有効な入力で成功() {
            let project = create_active_project();
            let command = Command { new_name: "新しい名前".to_string() };

            let result = run(project, command);

            assert!(result.is_ok());
            assert_eq!(result.unwrap().name(), "新しい名前");
        }

        #[test]
        fn 無効な入力でエラー() {
            let project = create_active_project();
            let command = Command { new_name: "".to_string() };

            let result = run(project, command);

            assert!(result.is_err());
        }
    }
}
```

---

## チェックリスト

ドメイン層のコードをレビューする際は以下を確認:

### 基本原則
- [ ] 外部クレート（sqlx, axum 等）への依存がない
- [ ] I/O 操作を行っていない
- [ ] イミュータブルな更新パターンを使用している

### モデル
- [ ] ID は NewType パターンで定義されている
- [ ] フィールドは private でゲッター経由でアクセス

### アクション
- [ ] 1アクション1ファイルになっている
- [ ] Command 構造体が定義されている
- [ ] validate / execute / run が分離されている
- [ ] validate は `Result<(), Error>` を返している
- [ ] execute は新しいモデルを返している（Result ではない）
- [ ] run は validate + execute を呼び出している
- [ ] エラー型は種類のみを定義している（メッセージを含まない）
- [ ] エラーに付随する情報（制限値など）はフィールドとして持たせている

### テスト
- [ ] validate のテストが記述されている
- [ ] execute のテストが記述されている
- [ ] run の統合テストが記述されている

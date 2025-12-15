# Use Case Layer Skill

## 概要

ユースケース層はドメインアクションを組み合わせてビジネスフローを実現するオーケストレーション層である。
このスキルはユースケース層の設計・実装ルールを定義する。

---

## 基本原則

### 依存の方向

ユースケース層は以下にのみ依存する：

| 依存先 | 目的 |
|--------|------|
| **domain 層** | ドメインモデルとアクションの呼び出し |
| **ports 層** | リポジトリトレイト（永続化の抽象） |

**禁止される依存:**
- repository 層（具体的な DB 実装）
- presentation 層（GraphQL、HTTP）
- infrastructure 層（接続プールなど）

### 責務

ユースケース層の責務は以下に限定する：

| やること | やらないこと |
|---------|-------------|
| ドメインアクションの呼び出し | ビジネスロジックの実装 |
| ports 経由の永続化 | 直接的な DB 操作 |
| トランザクション境界の管理 | SQL の記述 |
| DB 問い合わせが必要な検証 | ユーザー向けメッセージの生成 |
| フローの制御（条件分岐、順序） | HTTP/GraphQL の詳細 |

---

## ファイル配置

```
backend/src/use_case/
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

**1ユースケース1ファイル** の原則を採用する。
ファイル名はドメインアクションと同じ名前にする。

---

## ユースケースの実装パターン

### 基本構造

```rust
// backend/src/use_case/project/create_project.rs

use crate::domain::actions::project::create_project;
use crate::domain::models::project::Project;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースのエラー型
#[derive(Debug)]
pub enum Error {
    /// ドメインエラー（アクションのバリデーション失敗）
    Domain(create_project::Error),
    /// 重複エラー（DB 問い合わせが必要な検証）
    DuplicateName,
    /// インフラエラー（永続化失敗など）
    Infrastructure(String),
}

/// ユースケースへの入力
pub struct Input {
    pub name: String,
    pub description: Option<String>,
    pub goal: Option<String>,
}

/// ユースケースの実行
///
/// トランザクション管理:
/// - UnitOfWork がトランザクション境界を提供
/// - 正常終了時は自動コミット、エラー時は自動ロールバック
pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    input: Input,
) -> Result<Project, Error> {
    // 1. DB 問い合わせが必要な検証
    if uow.project_repository()
        .exists_by_name(&input.name)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?
    {
        return Err(Error::DuplicateName);
    }

    // 2. ドメインアクションの実行
    let command = create_project::Command {
        name: input.name,
        description: input.description,
        goal: input.goal,
    };
    let project = create_project::run(command).map_err(Error::Domain)?;

    // 3. 永続化
    uow.project_repository()
        .save(&project)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;

    // 4. コミット（明示的に呼び出す）
    uow.commit().await.map_err(|e| Error::Infrastructure(e.to_string()))?;

    Ok(project)
}
```

### 構造の解説

| 要素 | 役割 |
|------|------|
| `Error` | ユースケース固有のエラー型 |
| `Input` | ユースケースの入力（presentation 層から受け取る） |
| `execute` | ユースケースの実行関数 |
| `UnitOfWork` | トランザクション境界とリポジトリへのアクセスを提供 |

### UnitOfWork パターン

トランザクション管理には **UnitOfWork パターン** を採用する。

```rust
// ports/unit_of_work.rs

/// トランザクション境界を管理するトレイト
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type ProjectRepo: ProjectRepository;
    type TrialRepo: TrialRepository;
    type FeedbackRepo: FeedbackRepository;

    /// プロジェクトリポジトリを取得
    fn project_repository(&mut self) -> &mut Self::ProjectRepo;

    /// 試行リポジトリを取得
    fn trial_repository(&mut self) -> &mut Self::TrialRepo;

    /// フィードバックリポジトリを取得
    fn feedback_repository(&mut self) -> &mut Self::FeedbackRepo;

    /// トランザクションをコミット
    async fn commit(&mut self) -> Result<(), RepositoryError>;

    /// トランザクションをロールバック（明示的に呼び出す場合）
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}
```

**UnitOfWork を使う理由:**

| 観点 | メリット |
|------|---------|
| 一貫性 | 複数リポジトリの操作を1トランザクションで管理 |
| 明確性 | トランザクション境界がコード上で明確 |
| テスト容易性 | モック化してトランザクション動作を検証可能 |
| 責務分離 | ユースケースはフロー制御、UnitOfWork はトランザクション管理 |

---

## DB 問い合わせが必要な検証

### なぜユースケース層で行うか

ドメイン層は純粋関数であり、外部リソース（DB）にアクセスできない。
以下のような検証は DB への問い合わせが必要なため、ユースケース層で行う：

| 検証の種類 | 例 |
|-----------|-----|
| 一意性チェック | プロジェクト名の重複確認 |
| 存在チェック | 参照先エンティティの存在確認 |
| 整合性チェック | 他エンティティとの関連チェック |

### 検証の順序

1. **DB 問い合わせが必要な検証**（早期に失敗させる）
2. **ドメインアクションの実行**（純粋なバリデーション + 状態遷移）
3. **永続化**

この順序により、無駄な処理を最小限に抑える。

---

## エラー型の設計

### ユースケースのエラー型

ユースケース層のエラーは以下の3種類に分類する

| 種類 | 発生源 | 例 |
|------|--------|-----|
| Domain | ドメインアクションの validate | 名前が空、文字数超過 |
| ビジネスルール | ユースケース層の検証 | 名前重複、参照先不存在 |
| Infrastructure | ports の実装 | DB 接続失敗、タイムアウト |

### From トレイトの実装

ドメインエラーからの変換を容易にするため、From トレイトを実装する：

```rust
impl From<create_project::Error> for Error {
    fn from(e: create_project::Error) -> Self {
        Error::Domain(e)
    }
}
```

これにより `?` 演算子で自動変換できる：

```rust
let project = create_project::run(command)?; // 自動的に Error::Domain に変換
```

---

## トランザクション境界

### 基本方針

- ユースケースの `execute` 関数がトランザクション境界となる
- 1ユースケース = 1トランザクション を基本とする
- 複数エンティティを更新する場合も同一トランザクション内で行う
- **UnitOfWork** がトランザクションのライフサイクルを管理する

### トランザクションのライフサイクル

```
presentation 層: UnitOfWork を生成
        ↓
use_case 層: execute(uow, input) を実行
        ↓
    正常終了 → uow.commit()
    エラー発生 → uow.rollback()（または Drop で自動ロールバック）
```

### 実装パターン（複数エンティティの更新）

```rust
/// 試行を記録し、プロジェクトに紐づける
pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    input: Input,
) -> Result<Trial, Error> {
    // 1. プロジェクトの存在確認
    let project = uow.project_repository()
        .find_by_id(&input.project_id)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?
        .ok_or(Error::ProjectNotFound)?;

    // 2. ドメインアクション実行（試行を記録）
    let trial_command = record_trial::Command {
        project_id: project.id().clone(),
        parameters: input.parameters,
        notes: input.notes,
    };
    let trial = record_trial::run(trial_command).map_err(Error::Domain)?;

    // 3. Trial を保存
    uow.trial_repository()
        .save(&trial)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;

    // 4. Project に Trial を紐づけ（ドメインアクション）
    let add_command = add_trial_to_project::Command {
        trial_id: trial.id().clone(),
    };
    let updated_project = add_trial_to_project::run(project, add_command)
        .map_err(Error::Domain)?;

    // 5. 更新された Project を保存
    uow.project_repository()
        .save(&updated_project)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;

    // 6. コミット（すべての操作が成功した場合のみ）
    uow.commit().await.map_err(|e| Error::Infrastructure(e.to_string()))?;

    Ok(trial)
}
```

### エラー時の自動ロールバック

UnitOfWork の実装では、`commit()` が呼ばれずにドロップされた場合に自動でロールバックする：

```rust
// repository 層での UnitOfWork 実装例
impl Drop for SqlxUnitOfWork {
    fn drop(&mut self) {
        // commit されていない場合、トランザクションは自動でロールバック
        // （SQLx の Transaction は Drop 時に自動ロールバック）
    }
}
```

これにより、`?` でエラーが伝播した場合も安全にロールバックされる。

---

## 複数ドメインアクションのオーケストレーション

### いつ必要か

- 1つのユースケースで複数のモデルを更新する場合
- ドメインアクションの結果に基づいて別のアクションを実行する場合

### 実装パターン

複数エンティティの更新は「トランザクション境界」セクションの実装パターンを参照。
UnitOfWork により、すべての操作が同一トランザクション内で実行される。

### オーケストレーションの流れ

```
1. 必要なエンティティを取得（リポジトリ経由）
    ↓
2. ドメインアクション A を実行
    ↓
3. ドメインアクション B を実行（A の結果を使用）
    ↓
4. すべての変更を永続化
    ↓
5. コミット
```

**ポイント:**
- 各ドメインアクションは独立した純粋関数
- ユースケース層がアクションの実行順序を制御
- すべての永続化が成功してからコミット

---

## アンチパターン

### NG: ビジネスロジックをユースケースに実装

```rust
// ❌ バリデーションロジックをユースケースに直接記述
pub async fn execute<R: ProjectRepository>(
    repository: &R,
    input: Input,
) -> Result<Project, Error> {
    // ユースケースでバリデーション（ドメインの責務）
    if input.name.is_empty() {
        return Err(Error::EmptyName);
    }
    if input.name.len() > 100 {
        return Err(Error::NameTooLong);
    }

    let project = Project::new(input.name, input.description, input.goal);
    repository.save(&project).await?;
    Ok(project)
}

// ✅ ドメインアクションを呼び出す
pub async fn execute<R: ProjectRepository>(
    repository: &R,
    input: Input,
) -> Result<Project, Error> {
    // DB 問い合わせが必要な検証のみユースケースで行う
    if repository.exists_by_name(&input.name).await? {
        return Err(Error::DuplicateName);
    }

    // バリデーションはドメインアクションが担当
    let command = create_project::Command { ... };
    let project = create_project::run(command)?;

    repository.save(&project).await?;
    Ok(project)
}
```

### NG: 具体的な DB 操作

```rust
// ❌ SQL を直接記述
use sqlx::PgPool;

pub async fn execute(pool: &PgPool, input: Input) -> Result<Project, Error> {
    sqlx::query("INSERT INTO projects ...")
        .execute(pool)
        .await?;
    // ...
}

// ✅ UnitOfWork 経由で永続化
pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    input: Input,
) -> Result<Project, Error> {
    // ...
    uow.project_repository().save(&project).await?;
    uow.commit().await?;
    // ...
}
```

### NG: ユーザー向けメッセージの生成

```rust
// ❌ ユースケースでエラーメッセージを生成
pub enum Error {
    DuplicateName(String), // "プロジェクト名 'xxx' は既に存在します"
}

// ✅ エラーの種類のみを定義
pub enum Error {
    DuplicateName, // メッセージは presentation 層で変換
}
```

### NG: 永続化を忘れる

```rust
// ❌ ドメインアクション実行後、永続化していない
pub async fn execute<R: ProjectRepository>(
    repository: &R,
    input: Input,
) -> Result<Project, Error> {
    let project = create_project::run(command)?;
    // repository.save() を呼んでいない！
    Ok(project)
}

// ✅ 必ず永続化する
pub async fn execute<R: ProjectRepository>(
    repository: &R,
    input: Input,
) -> Result<Project, Error> {
    let project = create_project::run(command)?;
    repository.save(&project).await?;
    Ok(project)
}
```

### NG: 検証の順序が不適切

```rust
// ❌ ドメインアクション実行後に DB 検証（無駄な処理）
pub async fn execute<R: ProjectRepository>(
    repository: &R,
    input: Input,
) -> Result<Project, Error> {
    // ドメインアクション実行
    let project = create_project::run(command)?;

    // 後から重複チェック（既にドメイン処理が終わっている）
    if repository.exists_by_name(&project.name()).await? {
        return Err(Error::DuplicateName);
    }

    repository.save(&project).await?;
    Ok(project)
}

// ✅ DB 検証を先に行う
pub async fn execute<R: ProjectRepository>(
    repository: &R,
    input: Input,
) -> Result<Project, Error> {
    // 先に重複チェック（失敗すれば早期リターン）
    if repository.exists_by_name(&input.name).await? {
        return Err(Error::DuplicateName);
    }

    // ドメインアクション実行
    let project = create_project::run(command)?;

    repository.save(&project).await?;
    Ok(project)
}
```

---

## 更新系ユースケースのパターン

### 取得 → 更新 → 保存

```rust
pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    input: Input,
) -> Result<Project, Error> {
    // 1. 現在の状態を取得
    let project = uow.project_repository()
        .find_by_id(&input.project_id)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?
        .ok_or(Error::ProjectNotFound)?;

    // 2. DB 問い合わせが必要な検証（必要に応じて）
    if let Some(ref new_name) = input.new_name {
        if uow.project_repository()
            .exists_by_name_excluding_id(new_name, &input.project_id)
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))?
        {
            return Err(Error::DuplicateName);
        }
    }

    // 3. ドメインアクション実行
    let command = update_project_name::Command {
        new_name: input.new_name,
    };
    let updated_project = update_project_name::run(project, command)
        .map_err(Error::Domain)?;

    // 4. 永続化
    uow.project_repository()
        .save(&updated_project)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;

    // 5. コミット
    uow.commit().await.map_err(|e| Error::Infrastructure(e.to_string()))?;

    Ok(updated_project)
}
```

---

## テスト

### ユースケースのテスト方針

- **MockUnitOfWork** を使用してトランザクション動作を検証
- ドメインアクション自体のテストは domain 層で行う
- ユースケース層のテストはオーケストレーションの検証に集中

### テスト用の MockUnitOfWork

```rust
// tests/mocks/unit_of_work.rs

pub struct MockUnitOfWork {
    pub project_repo: MockProjectRepository,
    pub trial_repo: MockTrialRepository,
    pub feedback_repo: MockFeedbackRepository,
    pub committed: bool,
    pub rolled_back: bool,
}

impl MockUnitOfWork {
    pub fn new() -> Self {
        Self {
            project_repo: MockProjectRepository::new(),
            trial_repo: MockTrialRepository::new(),
            feedback_repo: MockFeedbackRepository::new(),
            committed: false,
            rolled_back: false,
        }
    }
}

#[async_trait]
impl UnitOfWork for MockUnitOfWork {
    type ProjectRepo = MockProjectRepository;
    type TrialRepo = MockTrialRepository;
    type FeedbackRepo = MockFeedbackRepository;

    fn project_repository(&mut self) -> &mut Self::ProjectRepo {
        &mut self.project_repo
    }

    fn trial_repository(&mut self) -> &mut Self::TrialRepo {
        &mut self.trial_repo
    }

    fn feedback_repository(&mut self) -> &mut Self::FeedbackRepo {
        &mut self.feedback_repo
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        self.committed = true;
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        self.rolled_back = true;
        Ok(())
    }
}
```

### テスト例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mocks::MockUnitOfWork;

    #[tokio::test]
    async fn 新規プロジェクトを作成できる() {
        // Arrange
        let mut uow = MockUnitOfWork::new();
        uow.project_repo.expect_exists_by_name()
            .returning(|_| Ok(false));
        uow.project_repo.expect_save()
            .returning(|_| Ok(()));

        let input = Input {
            name: "ピザ生地研究".to_string(),
            description: Some("加水率の研究".to_string()),
            goal: None,
        };

        // Act
        let result = execute(&mut uow, input).await;

        // Assert
        assert!(result.is_ok());
        assert!(uow.committed); // コミットされたことを確認
        let project = result.unwrap();
        assert_eq!(project.name(), "ピザ生地研究");
    }

    #[tokio::test]
    async fn 重複する名前でエラー_コミットされない() {
        // Arrange
        let mut uow = MockUnitOfWork::new();
        uow.project_repo.expect_exists_by_name()
            .returning(|_| Ok(true)); // 重複あり

        let input = Input {
            name: "既存プロジェクト".to_string(),
            description: None,
            goal: None,
        };

        // Act
        let result = execute(&mut uow, input).await;

        // Assert
        assert!(matches!(result, Err(Error::DuplicateName)));
        assert!(!uow.committed); // コミットされていないことを確認
    }

    #[tokio::test]
    async fn ドメインバリデーション失敗でエラー_コミットされない() {
        // Arrange
        let mut uow = MockUnitOfWork::new();
        uow.project_repo.expect_exists_by_name()
            .returning(|_| Ok(false));

        let input = Input {
            name: "".to_string(), // 空の名前
            description: None,
            goal: None,
        };

        // Act
        let result = execute(&mut uow, input).await;

        // Assert
        assert!(matches!(result, Err(Error::Domain(_))));
        assert!(!uow.committed); // コミットされていないことを確認
    }
}
```

---

## チェックリスト

ユースケース層のコードをレビューする際は以下を確認:

### 依存関係
- [ ] domain 層と ports 層にのみ依存している
- [ ] repository 層や infrastructure 層に直接依存していない
- [ ] SQL や DB 固有のコードがない

### 責務
- [ ] ドメインアクションを呼び出している（直接ロジックを実装していない）
- [ ] UnitOfWork 経由で永続化している
- [ ] ユーザー向けメッセージを生成していない

### エラー処理
- [ ] Error 型が定義されている
- [ ] ドメインエラー、ビジネスルールエラー、インフラエラーが分類されている
- [ ] From トレイトでドメインエラーを変換している

### 検証
- [ ] DB 問い合わせが必要な検証はドメインアクション実行前に行っている
- [ ] 純粋なバリデーションはドメインアクションに委譲している

### トランザクション管理
- [ ] UnitOfWork を引数として受け取っている
- [ ] すべての操作が成功した後に `commit()` を呼んでいる
- [ ] エラー時は `commit()` が呼ばれずに終了している（自動ロールバック）

### 永続化
- [ ] ドメインアクション実行後、永続化を忘れていない

### テスト
- [ ] MockUnitOfWork を使ったテストが記述されている
- [ ] 正常系と異常系のテストがある
- [ ] コミット/ロールバックの検証がある

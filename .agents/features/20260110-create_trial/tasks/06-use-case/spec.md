# Task: Use Case 実装

> Feature: [create_trial](../../spec.md)
> 依存: [03-domain-action](../03-domain-action/), [04-ports](../04-ports/)

## 目的

create_trial ユースケースを実装する。ドメインアクションを呼び出し、UnitOfWork 経由で永続化する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/use_case.rs` | 修正 | trial モジュールの追加 |
| `src/use_case/trial.rs` | 新規 | trial ユースケースモジュール |
| `src/use_case/trial/create_trial.rs` | 新規 | create_trial ユースケース |
| `src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository の追加 |

---

## 設計詳細

### ファイル構成

```
src/use_case/
├── project.rs      (既存)
├── project/        (既存)
├── trial.rs        (新規)
├── trial/
│   └── create_trial.rs  (新規)
└── test/
    └── mock_unit_of_work.rs  (修正)
```

### create_trial ユースケース

```rust
// src/use_case/trial/create_trial.rs

use crate::domain::actions::trial::create_trial as action;
use crate::domain::models::{Parameter, ParameterContent, ProjectId, Step, Trial};
use crate::ports::unit_of_work::UnitOfWork;

#[derive(Debug)]
pub enum Error {
    /// ドメインアクションのエラー
    Domain(action::Error),
    /// 指定された Project が存在しない
    ProjectNotFound,
    /// インフラエラー
    Infrastructure(String),
}

pub struct Input {
    pub project_id: ProjectId,
    pub memo: Option<String>,
    pub steps: Vec<StepInput>,
}

pub struct StepInput {
    pub name: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Vec<ParameterInput>,
}

pub struct ParameterInput {
    pub content: ParameterContent,
}

pub struct Output {
    pub trial: Trial,
    pub steps: Vec<Step>,
    pub parameters: Vec<Parameter>,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Output, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. Project の存在確認（DB 検証を先に）
    let project_exists = uow
        .project_repository()
        .find_by_id(&input.project_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
        .is_some();

    if !project_exists {
        let _ = uow.rollback().await;
        return Err(Error::ProjectNotFound);
    }

    // 3. Input → Command 変換
    let command = action::Command {
        project_id: input.project_id,
        memo: input.memo,
        steps: input
            .steps
            .into_iter()
            .map(|s| action::StepCommand {
                name: s.name,
                started_at: s.started_at,
                parameters: s
                    .parameters
                    .into_iter()
                    .map(|p| action::ParameterCommand { content: p.content })
                    .collect(),
            })
            .collect(),
    };

    // 4. ドメインアクション実行
    let action_output = match action::run(command) {
        Ok(output) => output,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 5. 永続化
    if let Err(e) = uow
        .trial_repository()
        .save_all(
            &action_output.trial,
            &action_output.steps,
            &action_output.parameters,
        )
        .await
    {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 6. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(Output {
        trial: action_output.trial,
        steps: action_output.steps,
        parameters: action_output.parameters,
    })
}
```

### MockTrialRepository の追加

```rust
// src/use_case/test/mock_unit_of_work.rs

pub struct MockTrialRepository {
    trials: Arc<Mutex<Vec<Trial>>>,
    steps: Arc<Mutex<Vec<Step>>>,
    parameters: Arc<Mutex<Vec<Parameter>>>,
}

#[async_trait]
impl TrialRepository for MockTrialRepository {
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError> {
        let trials = self.trials.lock().await;
        Ok(trials.iter().find(|t| t.id() == id).cloned())
    }

    async fn save_all(
        &self,
        trial: &Trial,
        steps: &[Step],
        parameters: &[Parameter],
    ) -> Result<(), RepositoryError> {
        self.trials.lock().await.push(trial.clone());
        self.steps.lock().await.extend(steps.iter().cloned());
        self.parameters.lock().await.extend(parameters.iter().cloned());
        Ok(())
    }

    // ... 他のメソッド
}
```

---

## テストケース

### テストファイル

- **ユニットテスト**: `src/use_case/trial/create_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_success` | 正常に Trial を作成できる |
| `test_create_trial_with_all_parameter_types` | 全種類の Parameter で作成できる |
| `test_output_contains_all_entities` | Output に Trial, Steps, Parameters が含まれる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_project_not_found` | Project が存在しない場合 ProjectNotFound エラー |
| `test_returns_domain_error_when_steps_empty` | Steps が空の場合 Domain エラー |
| `test_returns_domain_error_when_parameter_invalid` | Parameter が不正な場合 Domain エラー |
| `test_rollback_on_project_not_found` | ProjectNotFound 時にロールバックされる |
| `test_rollback_on_domain_error` | Domain エラー時にロールバックされる |

---

## 完了条件

- [ ] create_trial ユースケースが実装されている
- [ ] Input, StepInput, ParameterInput, Output が定義されている
- [ ] Error 型が Domain, ProjectNotFound, Infrastructure を持つ
- [ ] MockTrialRepository が追加されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

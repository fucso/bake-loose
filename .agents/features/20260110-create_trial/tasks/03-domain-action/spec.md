# Task: ドメインアクション

> Feature: [create_trial](../../spec.md)
> 依存: [02-domain-models](../02-domain-models/)

## 目的

Trial 作成のドメインアクション（create_trial）を実装する。validate / execute / run パターンに従う。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/domain/actions.rs` | 修正 | trial モジュールの追加 |
| `src/domain/actions/trial.rs` | 新規 | trial アクションモジュール |
| `src/domain/actions/trial/create_trial.rs` | 新規 | create_trial アクション |

---

## 設計詳細

### ファイル構成

```
src/domain/actions/
├── project.rs      (既存)
├── project/        (既存)
├── trial.rs        (新規)
└── trial/
    └── create_trial.rs  (新規)
```

### create_trial アクション

Trial を作成する。入力として Steps と各 Step の Parameters を受け取り、一括で構築する。

**Command 構造:**

```rust
// src/domain/actions/trial/create_trial.rs

pub struct Command {
    pub project_id: ProjectId,
    pub memo: Option<String>,
    pub steps: Vec<StepCommand>,
}

pub struct StepCommand {
    pub name: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Vec<ParameterCommand>,
}

pub struct ParameterCommand {
    pub content: ParameterContent,
}
```

**エラー型:**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Steps が空
    EmptySteps,
    /// Step 内の Parameter でバリデーションエラー
    InvalidParameter {
        step_index: usize,
        parameter_index: usize,
        reason: ParameterValidationError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValidationError {
    /// KeyValue の key が空
    EmptyKey,
    /// KeyValue の Text 値が空
    EmptyTextValue,
    /// TextParameter の value が空
    EmptyText,
    /// TimePoint の note が空
    EmptyTimePointNote,
}
```

**validate 関数:**

```rust
pub fn validate(command: &Command) -> Result<(), Error> {
    // Steps が空でないことを確認
    if command.steps.is_empty() {
        return Err(Error::EmptySteps);
    }

    // 各 Step の各 Parameter を検証
    for (step_idx, step) in command.steps.iter().enumerate() {
        for (param_idx, param) in step.parameters.iter().enumerate() {
            if let Err(reason) = validate_parameter(&param.content) {
                return Err(Error::InvalidParameter {
                    step_index: step_idx,
                    parameter_index: param_idx,
                    reason,
                });
            }
        }
    }

    Ok(())
}

fn validate_parameter(content: &ParameterContent) -> Result<(), ParameterValidationError> {
    match content {
        ParameterContent::KeyValue(kv) => {
            if kv.key.trim().is_empty() {
                return Err(ParameterValidationError::EmptyKey);
            }
            if let ParameterValue::Text(text) = &kv.value {
                if text.trim().is_empty() {
                    return Err(ParameterValidationError::EmptyTextValue);
                }
            }
        }
        ParameterContent::Text(t) => {
            if t.value.trim().is_empty() {
                return Err(ParameterValidationError::EmptyText);
            }
        }
        ParameterContent::TimePoint(tp) => {
            if tp.note.trim().is_empty() {
                return Err(ParameterValidationError::EmptyTimePointNote);
            }
        }
        ParameterContent::DurationRange(_) => {
            // DurationRange は note が Option なのでバリデーション不要
        }
    }
    Ok(())
}
```

**execute 関数:**

```rust
pub struct Output {
    pub trial: Trial,
    pub steps: Vec<Step>,
    pub parameters: Vec<Parameter>,
}

pub fn execute(command: Command) -> Output {
    let trial = Trial::new(command.project_id, command.memo);
    let trial_id = trial.id().clone();

    let mut steps = Vec::new();
    let mut parameters = Vec::new();

    for (position, step_cmd) in command.steps.into_iter().enumerate() {
        let step = Step::new(
            trial_id.clone(),
            position as u32,
            step_cmd.name,
            step_cmd.started_at,
        );
        let step_id = step.id().clone();

        for param_cmd in step_cmd.parameters {
            let parameter = Parameter::new(step_id.clone(), param_cmd.content);
            parameters.push(parameter);
        }

        steps.push(step);
    }

    Output {
        trial,
        steps,
        parameters,
    }
}
```

**run 関数:**

```rust
pub fn run(command: Command) -> Result<Output, Error> {
    validate(&command)?;
    Ok(execute(command))
}
```

---

## テストケース

### テストファイル

- **ユニットテスト**: `src/domain/actions/trial/create_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_with_single_step` | 1 つの Step で Trial を作成できる |
| `test_create_trial_with_multiple_steps` | 複数の Step で Trial を作成できる |
| `test_create_trial_with_all_parameter_types` | 全種類の Parameter を持つ Trial を作成できる |
| `test_step_positions_are_sequential` | Step の position が 0 から順番に設定される |
| `test_trial_status_is_in_progress` | 作成直後の Trial は InProgress |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_steps_empty` | Steps が空の場合 EmptySteps エラー |
| `test_returns_error_when_key_value_key_empty` | KeyValue の key が空の場合エラー |
| `test_returns_error_when_key_value_text_empty` | KeyValue の Text 値が空の場合エラー |
| `test_returns_error_when_text_parameter_empty` | TextParameter が空の場合エラー |
| `test_returns_error_when_time_point_note_empty` | TimePoint の note が空の場合エラー |
| `test_error_contains_step_and_parameter_index` | エラーに step_index と parameter_index が含まれる |

---

## 完了条件

- [ ] create_trial アクションが validate / execute / run パターンで実装されている
- [ ] Command, StepCommand, ParameterCommand が定義されている
- [ ] Error, ParameterValidationError が定義されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

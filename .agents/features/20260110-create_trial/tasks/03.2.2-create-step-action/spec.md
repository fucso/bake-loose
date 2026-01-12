# Task: Step 作成アクション

> Feature: [create_trial](../../spec.md)
> 依存: [03.2.1-aggregate-models](../03.2.1-aggregate-models/)

## 目的

Step と Parameter を作成するドメインアクション（create_step）を実装する。validate / execute / run パターンに従う。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial.rs` | 修正 | create_step モジュールの追加 |
| `backend/src/domain/actions/trial/create_step.rs` | 新規 | create_step アクション |

---

## 設計詳細

### ファイル構成

```
backend/src/domain/actions/
├── project.rs      (既存)
├── project/        (既存)
├── trial.rs        (既存 - 修正)
└── trial/
    ├── create_trial.rs  (既存)
    └── create_step.rs   (新規)
```

### create_step アクション

Trial に Step を追加する。Step には Parameters を含めることができる。

**Command 構造:**

```rust
// backend/src/domain/actions/trial/create_step.rs

pub struct Command {
    pub trial: Trial,
    pub name: Option<String>,
    pub started_at: DateTime<Utc>,
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
    /// Parameter でバリデーションエラー
    InvalidParameter {
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
    /// DurationRange の note が空
    EmptyDurationRangeNote,
}
```

**validate 関数:**

```rust
pub fn validate(command: &Command) -> Result<(), Error> {
    // 各 Parameter を検証
    for (param_idx, param) in command.parameters.iter().enumerate() {
        if let Err(reason) = validate_parameter(&param.content) {
            return Err(Error::InvalidParameter {
                parameter_index: param_idx,
                reason,
            });
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
            match &kv.value {
                ParameterValue::Text(text) => {
                    if text.trim().is_empty() {
                        return Err(ParameterValidationError::EmptyTextValue);
                    }
                }
                ParameterValue::Quantity { .. } => {
                    // amount と unit は型レベルで必須なのでOK
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
        ParameterContent::DurationRange(dr) => {
            if dr.note.trim().is_empty() {
                return Err(ParameterValidationError::EmptyDurationRangeNote);
            }
        }
    }
    Ok(())
}
```

**execute 関数:**

```rust
pub struct Output {
    pub trial: Trial,
}

pub fn execute(command: Command) -> Output {
    let mut trial = command.trial;

    // Trial から次の position を取得
    let next_position = trial.next_step_position();

    // Step を作成（started_at は必須）
    let mut step = Step::new(next_position, command.started_at);
    // name は Step のセッターで設定（実装に応じて調整）

    // Parameters を作成して Step に追加
    for param_cmd in command.parameters {
        let parameter = Parameter::new(param_cmd.content);
        step.add_parameter(parameter);
    }

    // Step を Trial に追加
    trial.add_step(step);

    Output { trial }
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

## 備考

### position の自動計算

position は `Trial::next_step_position()` を使って自動計算されます。

- 既存 Steps が空の場合: position = 0
- 既存 Steps がある場合: position = max(existing_positions) + 1

### started_at について

`started_at` は必須フィールドです。Step がいつ開始されたかを記録します。
呼び出し側（UseCase 層やプレゼンテーション層）で現在時刻を渡すことを想定しています。

### 集約の整合性

Trial を受け取り、Step を追加した Trial を返すことで、集約の整合性を保ちます。
UseCase 層では返された Trial を永続化します。

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/create_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_step_without_parameters` | Parameter なしで Step を作成できる |
| `test_create_step_with_parameters` | Parameter 付きで Step を作成できる |
| `test_create_step_with_all_parameter_types` | 全種類の Parameter を持つ Step を作成できる |
| `test_first_step_has_position_zero` | 空の Trial に追加すると position = 0 |
| `test_step_position_is_max_plus_one` | Steps がある Trial に追加すると position = max + 1 |
| `test_step_is_added_to_trial` | Step が Trial の steps に追加される |
| `test_step_has_started_at` | Step に started_at が設定される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_key_value_key_empty` | KeyValue の key が空の場合エラー |
| `test_returns_error_when_key_value_text_empty` | KeyValue の Text 値が空の場合エラー |
| `test_returns_error_when_text_parameter_empty` | TextParameter が空の場合エラー |
| `test_returns_error_when_time_point_note_empty` | TimePoint の note が空の場合エラー |
| `test_returns_error_when_duration_range_note_empty` | DurationRange の note が空の場合エラー |
| `test_error_contains_parameter_index` | エラーに parameter_index が含まれる |

---

## 完了条件

- [ ] create_step アクションが validate / execute / run パターンで実装されている
- [ ] Command が Trial をモデルで受け取る
- [ ] Command の started_at が必須（DateTime<Utc>）である
- [ ] position は Trial::next_step_position() で自動設定される
- [ ] Step が Trial に追加された状態で返される
- [ ] ParameterCommand が定義されている
- [ ] Error, ParameterValidationError が定義されている
- [ ] DurationRange の note が空の場合はエラーになる
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

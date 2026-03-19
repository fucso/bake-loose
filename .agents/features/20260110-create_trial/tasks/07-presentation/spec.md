# Task: Presentation 実装

> Feature: [create_trial](../../spec.md)
> 依存: [06-use-case](../06-use-case/)

## 目的

Trial 作成のための GraphQL Mutation と関連する型を実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/presentation/graphql/types.rs` | 修正 | 新規型モジュールの追加 |
| `src/presentation/graphql/types/trial.rs` | 新規 | Trial GraphQL 型 |
| `src/presentation/graphql/types/step.rs` | 新規 | Step GraphQL 型 |
| `src/presentation/graphql/types/parameter.rs` | 新規 | Parameter GraphQL 型 |
| `src/presentation/graphql/types/unit.rs` | 新規 | Unit, TimeUnit GraphQL 型 |
| `src/presentation/graphql/mutation.rs` | 修正 | TrialMutation の追加 |
| `src/presentation/graphql/mutation/trial.rs` | 新規 | createTrial Mutation |
| `src/presentation/graphql/error.rs` | 修正 | create_trial エラー変換 |

---

## 設計詳細

### GraphQL スキーマ（概要）

```graphql
# Types
type Trial {
  id: ID!
  projectId: ID!
  status: TrialStatus!
  memo: String
  steps: [Step!]!
}

enum TrialStatus {
  IN_PROGRESS
  COMPLETED
}

type Step {
  id: ID!
  name: String
  position: Int!
  startedAt: DateTime
  parameters: [Parameter!]!
}

type Parameter {
  id: ID!
  content: ParameterContent!
}

union ParameterContent =
  KeyValueParameter |
  TextParameter |
  DurationRangeParameter |
  TimePointParameter

type KeyValueParameter {
  key: String!
  value: ParameterValue!
}

union ParameterValue = TextValue | QuantityValue

type TextValue {
  text: String!
}

type QuantityValue {
  amount: Int!
  unit: Unit!
}

enum Unit {
  GRAM
  KILOGRAM
  CELSIUS
  MILLILITER
  LITER
  PERCENT
}

type TextParameter {
  value: String!
}

type DurationRangeParameter {
  durationSeconds: Int!
  displayUnit: TimeUnit!
  note: String
}

type TimePointParameter {
  elapsedSeconds: Int!
  displayUnit: TimeUnit!
  note: String!
}

enum TimeUnit {
  SECOND
  MINUTE
  HOUR
}

# Input types
input CreateTrialInput {
  projectId: ID!
  memo: String
  steps: [StepInput!]!
}

input StepInput {
  name: String
  startedAt: DateTime
  parameters: [ParameterInput!]!
}

input ParameterInput {
  keyValue: KeyValueInput
  text: TextInput
  durationRange: DurationRangeInput
  timePoint: TimePointInput
}

input KeyValueInput {
  key: String!
  textValue: String
  quantity: QuantityInput
}

input QuantityInput {
  amount: Int!
  unit: Unit!
}

input TextInput {
  value: String!
}

input DurationRangeInput {
  durationSeconds: Int!
  displayUnit: TimeUnit!
  note: String
}

input TimePointInput {
  elapsedSeconds: Int!
  displayUnit: TimeUnit!
  note: String!
}

# Mutation
type Mutation {
  createTrial(input: CreateTrialInput!): CreateTrialPayload!
}

type CreateTrialPayload {
  trial: Trial!
}
```

### Trial GraphQL 型

```rust
// src/presentation/graphql/types/trial.rs

pub struct Trial {
    trial: DomainTrial,
    steps: Vec<DomainStep>,
    parameters: Vec<DomainParameter>,
}

#[Object]
impl Trial {
    async fn id(&self) -> ID {
        ID(self.trial.id().0.to_string())
    }

    async fn project_id(&self) -> ID {
        ID(self.trial.project_id().0.to_string())
    }

    async fn status(&self) -> TrialStatus {
        self.trial.status().into()
    }

    async fn memo(&self) -> Option<&str> {
        self.trial.memo()
    }

    async fn steps(&self) -> Vec<Step> {
        // steps と parameters を組み合わせて Step 型を構築
        self.steps
            .iter()
            .map(|s| {
                let params: Vec<_> = self
                    .parameters
                    .iter()
                    .filter(|p| p.step_id() == s.id())
                    .cloned()
                    .collect();
                Step::new(s.clone(), params)
            })
            .collect()
    }
}
```

### Input 型から ParameterContent への変換

```rust
// src/presentation/graphql/mutation/trial.rs

impl TryFrom<ParameterInput> for ParameterContent {
    type Error = &'static str;

    fn try_from(input: ParameterInput) -> Result<Self, Self::Error> {
        // 排他的に 1 つだけ指定されていることを確認
        let count = [
            input.key_value.is_some(),
            input.text.is_some(),
            input.duration_range.is_some(),
            input.time_point.is_some(),
        ]
        .iter()
        .filter(|&&b| b)
        .count();

        if count != 1 {
            return Err("Exactly one parameter type must be specified");
        }

        if let Some(kv) = input.key_value {
            // KeyValue 変換
            let value = if let Some(text) = kv.text_value {
                ParameterValue::Text(text)
            } else if let Some(q) = kv.quantity {
                ParameterValue::Quantity {
                    amount: q.amount,
                    unit: q.unit.into(),
                }
            } else {
                return Err("KeyValue must have either textValue or quantity");
            };
            return Ok(ParameterContent::KeyValue(KeyValueParameter {
                key: kv.key,
                value,
            }));
        }

        // 他の型も同様に変換...
    }
}
```

### Mutation 実装

```rust
// src/presentation/graphql/mutation/trial.rs

#[derive(Default)]
pub struct TrialMutation;

#[Object]
impl TrialMutation {
    async fn create_trial(
        &self,
        ctx: &Context<'_>,
        input: CreateTrialInput,
    ) -> Result<CreateTrialPayload> {
        let mut uow = ctx.create_unit_of_work()?;

        // Input → use_case::Input 変換
        let use_case_input = use_case::trial::create_trial::Input {
            project_id: ProjectId(Uuid::parse_str(&input.project_id.0)?),
            memo: input.memo,
            steps: input
                .steps
                .into_iter()
                .map(|s| use_case::trial::create_trial::StepInput {
                    name: s.name,
                    started_at: s.started_at,
                    parameters: s
                        .parameters
                        .into_iter()
                        .map(|p| use_case::trial::create_trial::ParameterInput {
                            content: p.try_into()?,
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                })
                .collect::<Result<Vec<_>, _>>()?,
        };

        let output = use_case::trial::create_trial::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(CreateTrialPayload {
            trial: Trial::new(output.trial, output.steps, output.parameters),
        })
    }
}
```

### エラー変換

```rust
// src/presentation/graphql/error.rs

impl UserFacingError for use_case::trial::create_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            Self::ProjectNotFound => {
                GraphQLError::new("指定されたプロジェクトが見つかりません", "NOT_FOUND")
            }
            Self::Domain(e) => e.to_user_facing(),
            Self::Infrastructure(_) => {
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl UserFacingError for domain::actions::trial::create_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            Self::EmptySteps => {
                GraphQLError::new("少なくとも1つのステップを追加してください", "VALIDATION_ERROR")
            }
            Self::InvalidParameter { step_index, parameter_index, reason } => {
                let message = match reason {
                    ParameterValidationError::EmptyKey => "キーを入力してください",
                    ParameterValidationError::EmptyTextValue => "値を入力してください",
                    ParameterValidationError::EmptyText => "テキストを入力してください",
                    ParameterValidationError::EmptyTimePointNote => "メモを入力してください",
                };
                GraphQLError::new(
                    format!("ステップ{}のパラメーター{}: {}", step_index + 1, parameter_index + 1, message),
                    "VALIDATION_ERROR",
                )
            }
        }
    }
}
```

### MutationRoot への追加

```rust
// src/presentation/graphql/mutation.rs

#[derive(MergedObject, Default)]
pub struct MutationRoot(ProjectMutation, TrialMutation);
```

---

## テストケース

### テストファイル

- **統合テスト**: `tests/graphql/trial/create_trial.rs`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_returns_trial` | Trial が正常に作成される |
| `test_create_trial_with_key_value_text` | KeyValue (Text) パラメーターで作成 |
| `test_create_trial_with_key_value_quantity` | KeyValue (Quantity) パラメーターで作成 |
| `test_create_trial_with_text_parameter` | TextParameter で作成 |
| `test_create_trial_with_duration_range` | DurationRangeParameter で作成 |
| `test_create_trial_with_time_point` | TimePointParameter で作成 |
| `test_create_trial_with_multiple_steps` | 複数ステップで作成 |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_project_not_found` | 存在しない Project ID で NOT_FOUND |
| `test_returns_error_when_steps_empty` | Steps が空で VALIDATION_ERROR |
| `test_returns_error_when_key_empty` | KeyValue の key が空で VALIDATION_ERROR |
| `test_returns_error_when_multiple_parameter_types` | 複数のパラメーター型を同時指定でエラー |

---

## 完了条件

- [ ] Trial, Step, Parameter の GraphQL 型が定義されている
- [ ] Input 型（CreateTrialInput 等）が定義されている
- [ ] createTrial Mutation が実装されている
- [ ] エラー変換が実装されている
- [ ] MutationRoot に TrialMutation が追加されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

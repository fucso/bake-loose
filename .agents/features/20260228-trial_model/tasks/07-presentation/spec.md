# Task: GraphQL 実装

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 05-repository, 06.1〜06.6

## 目的

Trial 関連の GraphQL スキーマ・リゾルバーを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/presentation/graphql/types/trial.rs` | 新規 | Trial, Step, Parameter 型 |
| `backend/src/presentation/graphql/types/parameter_content.rs` | 新規 | ParameterContent union |
| `backend/src/presentation/graphql/types.rs` | 修正 | モジュール追加 |
| `backend/src/presentation/graphql/query/trial.rs` | 新規 | Trial クエリ |
| `backend/src/presentation/graphql/query.rs` | 修正 | モジュール追加 |
| `backend/src/presentation/graphql/mutation/trial.rs` | 新規 | Trial ミューテーション |
| `backend/src/presentation/graphql/mutation.rs` | 修正 | モジュール追加 |
| `backend/src/presentation/graphql/error.rs` | 修正 | Trial 関連エラー変換追加 |
| `backend/src/presentation/graphql/schema.rs` | 修正 | スキーマに追加 |

---

## 設計詳細

### GraphQL 型

#### Trial

```graphql
type Trial {
  id: ID!
  projectId: ID!
  name: String
  memo: String
  status: TrialStatus!
  steps: [Step!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

enum TrialStatus {
  IN_PROGRESS
  COMPLETED
}
```

#### Step

```graphql
type Step {
  id: ID!
  name: String!
  position: Int!
  startedAt: DateTime
  completedAt: DateTime
  parameters: [Parameter!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}
```

#### Parameter

```graphql
type Parameter {
  id: ID!
  content: ParameterContent!
  createdAt: DateTime!
  updatedAt: DateTime!
}

union ParameterContent = KeyValueParameter | DurationParameter | TimeMarkerParameter | TextParameter

type KeyValueParameter {
  key: String!
  value: ParameterValue!
}

union ParameterValue = TextValue | QuantityValue

type TextValue {
  value: String!
}

type QuantityValue {
  amount: Float!
  unit: String!
}

type DurationParameter {
  duration: DurationValue!
  note: String
}

type TimeMarkerParameter {
  at: DurationValue!
  note: String!
}

type DurationValue {
  value: Float!
  unit: String!
}

type TextParameter {
  value: String!
}
```

### Query

```graphql
type Query {
  trial(id: ID!): Trial
  trialsByProject(projectId: ID!): [Trial!]!
}
```

### Mutation

```graphql
type Mutation {
  createTrial(input: CreateTrialInput!): Trial!
  updateTrial(input: UpdateTrialInput!): Trial!
  completeTrial(id: ID!): Trial!
  addStep(input: AddStepInput!): Trial!
  updateStep(input: UpdateStepInput!): Trial!
  completeStep(input: CompleteStepInput!): Trial!
}

input CreateTrialInput {
  projectId: ID!
  name: String
  memo: String
  steps: [StepInput!]
}

input StepInput {
  name: String!
  startedAt: DateTime
  parameters: [ParameterInput!]
}

input ParameterInput {
  keyValue: KeyValueInput
  duration: DurationInput
  timeMarker: TimeMarkerInput
  text: String
}

input KeyValueInput {
  key: String!
  textValue: String
  quantity: QuantityInput
}

input QuantityInput {
  amount: Float!
  unit: String!
}

input DurationInput {
  value: Float!
  unit: String!
  note: String
}

input TimeMarkerInput {
  value: Float!
  unit: String!
  note: String!
}

input UpdateTrialInput {
  id: ID!
  name: String
  memo: String
}

input AddStepInput {
  trialId: ID!
  name: String!
  startedAt: DateTime
  parameters: [ParameterInput!]
}

input UpdateStepInput {
  trialId: ID!
  stepId: ID!
  name: String
  startedAt: DateTime
  addParameters: [ParameterInput!]
  removeParameterIds: [ID!]
}

input CompleteStepInput {
  trialId: ID!
  stepId: ID!
  completedAt: DateTime
}
```

### エラー変換

各ユースケースのエラーに対してユーザー向けメッセージを定義:

| エラー | コード | メッセージ |
|--------|--------|------------|
| ProjectNotFound | NOT_FOUND | プロジェクトが見つかりません |
| TrialNotFound | NOT_FOUND | トライアルが見つかりません |
| AlreadyCompleted | VALIDATION_ERROR | 既に完了しています |
| EmptyStepName | VALIDATION_ERROR | ステップ名を入力してください |
| InvalidParameter | VALIDATION_ERROR | パラメーターが不正です |

---

## テストケース

### テストファイル

- **統合テスト**: `tests/graphql/trial/` 配下

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial` | Trial を作成できる |
| `test_query_trial` | Trial を取得できる |
| `test_query_trials_by_project` | プロジェクト別に Trial を取得できる |
| `test_add_step` | Step を追加できる |
| `test_update_step` | Step を更新できる |
| `test_complete_step` | Step を完了できる |
| `test_complete_trial` | Trial を完了できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_with_invalid_project` | 存在しないプロジェクトでエラー |
| `test_update_completed_trial_returns_error` | 完了済み Trial の更新でエラー |

---

## 完了条件

- [ ] Trial, Step, Parameter 型が定義されている
- [ ] ParameterContent union が定義されている
- [ ] Query リゾルバーが実装されている
- [ ] Mutation リゾルバーが実装されている
- [ ] エラー変換が実装されている
- [ ] スキーマに追加されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

# Task: create_trial アクション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

新しい Trial を作成するドメインアクションを実装する。Trial は InProgress ステータスで作成され、初期の Steps とともに作成可能。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/create_trial.rs` | 新規 | create_trial アクション |
| `backend/src/domain/actions/trial.rs` | 新規 | trial アクションモジュール |
| `backend/src/domain/actions.rs` | 修正 | trial モジュール追加 |

---

## 設計詳細

### Command

- `project_id`: ProjectId - 所属プロジェクト（必須）
- `name`: Option<String> - 任意の名前
- `memo`: Option<String> - 備考・ノート
- `steps`: Vec<StepInput> - 初期ステップ（空でも可）

### StepInput

- `name`: String - ステップ名
- `started_at`: Option<DateTime<Utc>> - 開始日時
- `parameters`: Vec<ParameterInput> - パラメーター

### ParameterInput

- `content`: ParameterContent - パラメーター内容

### Error

- `EmptyStepName { step_index: usize }` - ステップ名が空
- `InvalidParameter { step_index: usize, parameter_index: usize, reason: ParameterValidationError }` - パラメーターが不正

### ParameterValidationError

- `EmptyKey` - KeyValue の key が空
- `EmptyTextValue` - KeyValue の Text 値が空、または Text 型の value が空
- `EmptyTimeMarkerNote` - TimeMarker の note が空
- `InvalidQuantity` - Quantity の amount が負数

### ロジック

1. 各 Step の name が空でないことを検証
2. 各 Step 内の Parameter を検証
   - KeyValue: key が空でないこと、Text 値の場合は値も空でないこと
   - TimeMarker: note が空でないこと
   - Quantity: amount が 0 以上であること
3. Trial を InProgress ステータスで作成
4. Steps の position を 0 から順番に自動付与
5. 作成した Trial を返す

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/create_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_with_no_steps` | Steps が空で Trial を作成できる |
| `test_create_trial_with_single_step` | 1 つの Step で Trial を作成できる |
| `test_create_trial_with_multiple_steps` | 複数の Steps で Trial を作成できる |
| `test_step_positions_are_sequential` | Step の position が 0 から順番に設定される |
| `test_create_trial_with_parameters` | Parameter を含む Step で Trial を作成できる |
| `test_trial_status_is_in_progress` | 作成された Trial のステータスが InProgress |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_step_name_empty` | Step 名が空の場合 EmptyStepName エラー |
| `test_error_contains_step_index` | エラーに step_index が含まれる |
| `test_returns_error_when_key_value_key_empty` | KeyValue の key が空の場合エラー |
| `test_returns_error_when_time_marker_note_empty` | TimeMarker の note が空の場合エラー |
| `test_error_contains_parameter_index` | エラーに parameter_index が含まれる |

---

## 完了条件

- [ ] Command, Error, ParameterValidationError が定義されている
- [ ] validate / execute / run 関数が実装されている
- [ ] Step の position が自動付与される
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

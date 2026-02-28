# Task: update_step アクション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

既存の Step の名前・開始日時・パラメーターを更新するドメインアクションを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/update_step.rs` | 新規 | update_step アクション |
| `backend/src/domain/actions/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Command

- `step_id`: StepId - 更新対象の Step ID
- `name`: Option<String> - 新しい名前（None の場合は変更なし）
- `started_at`: Option<Option<DateTime<Utc>>> - 新しい開始日時（None の場合は変更なし、Some(None) の場合はクリア）
- `add_parameters`: Vec<ParameterInput> - 追加するパラメーター
- `remove_parameter_ids`: Vec<ParameterId> - 削除するパラメーター ID

### Error

- `TrialAlreadyCompleted` - Trial が既に完了している
- `StepNotFound` - 指定された Step が存在しない
- `StepAlreadyCompleted` - Step が既に完了している
- `EmptyStepName` - 名前が空文字
- `InvalidParameter { parameter_index: usize, reason: ParameterValidationError }` - パラメーターが不正
- `ParameterNotFound { parameter_id: ParameterId }` - 削除対象のパラメーターが存在しない

### ロジック

1. Trial のステータスが Completed の場合はエラー
2. 指定された Step が Trial 内に存在するか検証
3. Step が Completed の場合はエラー
4. name が Some かつ空文字の場合はエラー
5. add_parameters を検証
6. remove_parameter_ids が存在するパラメーターか検証
7. 各フィールドを更新
8. パラメーターを追加・削除
9. Step と Trial の updated_at を更新
10. 更新後の Trial を返す

### 注意点

- started_at は `Option<Option<DateTime>>` で None/Some(None)/Some(Some(value)) を区別
- パラメーターの更新は「追加」と「削除」のみ（既存パラメーターの内容変更は delete + add で対応）

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/update_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_update_step_name` | Step の名前を更新できる |
| `test_update_step_started_at` | Step の開始日時を更新できる |
| `test_clear_step_started_at` | Step の開始日時をクリアできる |
| `test_add_parameters` | パラメーターを追加できる |
| `test_remove_parameters` | パラメーターを削除できる |
| `test_add_and_remove_parameters` | 追加と削除を同時に行える |
| `test_step_updated_at_is_changed` | Step の updated_at が更新される |
| `test_trial_updated_at_is_changed` | Trial の updated_at が更新される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_completed` | 完了済み Trial の場合 TrialAlreadyCompleted エラー |
| `test_returns_error_when_step_not_found` | Step が存在しない場合 StepNotFound エラー |
| `test_returns_error_when_step_completed` | 完了済み Step の場合 StepAlreadyCompleted エラー |
| `test_returns_error_when_name_empty` | 名前が空文字の場合 EmptyStepName エラー |
| `test_returns_error_when_parameter_invalid` | 不正なパラメーターの場合 InvalidParameter エラー |
| `test_returns_error_when_parameter_not_found` | 削除対象のパラメーターが存在しない場合 ParameterNotFound エラー |

---

## 完了条件

- [ ] Command, Error が定義されている
- [ ] validate / execute / run 関数が実装されている
- [ ] パラメーターの追加・削除が正しく動作する
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

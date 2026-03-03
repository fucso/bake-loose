# Task Report: add_step アクション

> 実施日時: 2026-03-03
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/add_step.rs` | 新規 | add_step アクション |
| `backend/src/domain/actions/trial.rs` | 新規 | trial アクションモジュール |
| `backend/src/domain/actions.rs` | 修正 | trial モジュール追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo fmt`、`cargo clippy -- -D warnings` ともにエラーなし）

### テスト

成功（38 tests passed）

新規追加のテスト（add_step 関連 11件）:

正常系:
- `test_add_step_to_empty_trial`
- `test_add_step_position_is_zero_for_first`
- `test_add_step_position_increments`
- `test_add_step_with_parameters`
- `test_add_step_with_started_at`
- `test_trial_updated_at_is_changed`

異常系:
- `test_returns_error_when_trial_completed`
- `test_returns_error_when_step_name_empty`
- `test_returns_error_when_step_name_whitespace_only`
- `test_returns_error_when_parameter_invalid`
- `test_parameter_duration_with_empty_unit_is_invalid`

## コミット情報

- ハッシュ: b0cf3d2
- ブランチ: task/20260228-trial_model_03.4-domain-action-add-step

## 次タスクへの申し送り

- `add_step::run(trial: Trial, command: Command) -> Result<Trial, Error>` が実装済み
- `Command` は `name: String`、`started_at: Option<DateTime<Utc>>`、`parameters: Vec<ParameterInput>` を持つ
- `ParameterInput` は `content: ParameterContent` を持つ
- `Error` は `TrialAlreadyCompleted`、`EmptyStepName`、`InvalidParameter { parameter_index, reason }` の3バリアント
- `ParameterValidationError` は `EmptyKey`、`EmptyNote`、`EmptyText`、`EmptyUnit` の4バリアント
- position は既存 Steps の `position` フィールドの最大値+1（Steps が空の場合は 0）
- `Step::from_raw` を使用して parameters と started_at を含む Step を構築
- `Trial::from_raw` を使用して updated_at を更新した Trial を返す
- モジュールパス: `crate::domain::actions::trial::add_step`

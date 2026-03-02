# Task Report: create_trial アクション

> 実施日時: 2026-03-03 00:00
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/create_trial.rs` | 新規 | create_trial アクション（Command, Error, validate, execute, run、ユニットテスト）|
| `backend/src/domain/actions/trial.rs` | 新規 | trial アクションモジュール |
| `backend/src/domain/actions.rs` | 修正 | trial モジュール追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo clippy -- -D warnings`、`cargo fmt -- --check` ともにエラーなし）

### テスト

成功（38 tests passed）

新規追加のテスト:
- `test_create_trial_with_no_steps`
- `test_create_trial_with_single_step`
- `test_create_trial_with_multiple_steps`
- `test_step_positions_are_sequential`
- `test_create_trial_with_parameters`
- `test_trial_status_is_in_progress`
- `test_returns_error_when_step_name_empty`
- `test_error_contains_step_index`
- `test_returns_error_when_key_value_key_empty`
- `test_returns_error_when_time_marker_note_empty`
- `test_error_contains_parameter_index`

## コミット情報

- ハッシュ: 82bb60c
- ブランチ: task/20260228-trial_model_03.1-domain-action-create-trial

## 次タスクへの申し送り

- `create_trial::Command` は `project_id: ProjectId`, `name: Option<String>`, `memo: Option<String>`, `steps: Vec<StepInput>` を受け取る
- `create_trial::run(command)` で Trial を作成。バリデーション + 生成を一括実行
- `StepInput` は `name`, `started_at`, `parameters: Vec<ParameterInput>` を持つ
- `ParameterInput` は `content: ParameterContent` のみ
- Step の position は 0 から順番に自動付与
- `Error` は `EmptyStepName { step_index }` / `InvalidParameter { step_index, parameter_index, reason: ParameterValidationError }` の 2 バリアント
- `ParameterValidationError` は `EmptyKey` / `EmptyTextValue` / `EmptyTimeMarkerNote` / `InvalidQuantity` の 4 バリアント
- Step / Parameter の構築には `from_raw` を使用（`started_at` セットや parameters セットが必要なため）
- モジュールパス: `crate::domain::actions::trial::create_trial`

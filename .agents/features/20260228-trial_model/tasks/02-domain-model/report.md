# Task Report: ドメインモデル

> 実施日時: 2026-03-03 00:00
> 依存タスク: なし

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/models/trial.rs` | 新規 | Trial, TrialId, TrialStatus の定義 |
| `backend/src/domain/models/step.rs` | 新規 | Step, StepId の定義 |
| `backend/src/domain/models/parameter.rs` | 新規 | Parameter, ParameterId, ParameterContent, ParameterValue, DurationValue の定義 |
| `backend/src/domain/models.rs` | 修正 | trial, step, parameter モジュールを追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo clippy -- -D warnings` ともにエラーなし）

### テスト

成功（27 tests passed）

新規追加のテスト:
- `test_trial_id_new_generates_unique_ids`
- `test_trial_new_creates_with_in_progress_status`
- `test_step_id_new_generates_unique_ids`
- `test_parameter_content_key_value_with_quantity`
- `test_parameter_content_duration_with_note`
- `test_duration_value_creation`

## コミット情報

- ハッシュ: 5bc142b
- ブランチ: task/20260228-trial_model_02-domain-model

## 次タスクへの申し送り

- `Trial` は aggregate root。`steps: Vec<Step>` を内包し、`Step` は `parameters: Vec<Parameter>` を内包する
- `Trial::new(project_id, name, memo)` で新規作成（ステータスは自動的に `InProgress`、steps は空）
- `Trial::from_raw(...)` で永続化層からの復元に使用（`#[allow(clippy::too_many_arguments)]` を付与済み）
- `Step::new(trial_id, name, position)` で新規作成（started_at/completed_at は None、parameters は空）
- `Step::from_raw(...)` で永続化層からの復元に使用（`#[allow(clippy::too_many_arguments)]` を付与済み）
- `Parameter::new(step_id, content)` で新規作成
- `ParameterContent` は `KeyValue / Duration / TimeMarker / Text` の 4 バリアント
- `ParameterValue` は `Text / Quantity` の 2 バリアント
- `DurationValue` は `value: f64, unit: String` のシンプルな構造体
- `f64` を含む型は `Eq` を derive していない（`PartialEq` のみ）
- モジュール参照: `crate::domain::models::trial::TrialId`、`crate::domain::models::step::StepId` など

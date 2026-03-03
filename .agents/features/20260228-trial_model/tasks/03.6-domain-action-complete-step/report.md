# Task Report: complete_step アクション

> 実施日時: 2026-03-03
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial.rs` | 新規 | trial アクションモジュール宣言 |
| `backend/src/domain/actions/trial/complete_step.rs` | 新規 | complete_step アクション実装 |
| `backend/src/domain/actions.rs` | 修正 | trial モジュール追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo clippy -- -D warnings` エラーなし）

### テスト

成功（35 tests passed）

新規追加のテスト:
- `test_complete_step` - Step を完了できる
- `test_completed_at_is_set` - completed_at が設定される
- `test_complete_step_with_specified_time` - 指定した日時で完了できる
- `test_step_updated_at_is_changed` - Step の updated_at が更新される
- `test_trial_updated_at_is_changed` - Trial の updated_at が更新される
- `test_returns_error_when_trial_completed` - 完了済み Trial の場合 TrialAlreadyCompleted エラー
- `test_returns_error_when_step_not_found` - Step が存在しない場合 StepNotFound エラー
- `test_returns_error_when_step_already_completed` - 既に完了済みの場合 StepAlreadyCompleted エラー

## コミット情報

- ハッシュ: b7ab308
- ブランチ: task/20260228-trial_model_03.6-domain-action-complete-step

## 次タスクへの申し送り

- `complete_step::Command` は `step_id: StepId` と `completed_at: Option<DateTime<Utc>>` を持つ
- `complete_step::Error` は `TrialAlreadyCompleted / StepNotFound / StepAlreadyCompleted` の 3 バリアント
- `complete_step::run(state: Trial, command: Command) -> Result<Trial, Error>` で実行
- 完了日時は Command で指定可能（None の場合は現在時刻）
- Trial アクションモジュールは `crate::domain::actions::trial` に配置

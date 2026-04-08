# Task Report: complete_trial アクション

> 実施日時: 2026-03-03
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/complete_trial.rs` | 新規 | complete_trial アクション（validate / execute / run + テスト） |
| `backend/src/domain/actions/trial.rs` | 新規 | trial アクションモジュール定義 |
| `backend/src/domain/actions.rs` | 修正 | `pub mod trial;` を追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo clippy -- -D warnings` でエラー・警告なし）

### テスト

成功（32 tests passed）

新規追加のテスト:
- `test_complete_trial` - InProgress の Trial を完了できる
- `test_status_is_completed` - 完了後のステータスが Completed
- `test_updated_at_is_changed` - updated_at が更新される
- `test_complete_trial_with_incomplete_steps` - 未完了の Step があっても完了できる
- `test_returns_error_when_already_completed` - 既に完了済みの場合 AlreadyCompleted エラー

## コミット情報

- ハッシュ: 953906b
- ブランチ: task/20260228-trial_model_03.3-domain-action-complete-trial

## 次タスクへの申し送り

- `complete_trial::run(state: Trial) -> Result<Trial, Error>` が実装済み
- `Error::AlreadyCompleted` のみを定義（Command なし）
- `execute` は `Trial::from_raw()` で再構築し `updated_at` を `Utc::now()` に更新
- 未完了の Step があっても Trial の完了を許容（ビジネスルール上の決定）
- モジュール参照: `crate::domain::actions::trial::complete_trial`

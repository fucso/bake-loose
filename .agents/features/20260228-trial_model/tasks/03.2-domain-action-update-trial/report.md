# Task Report: update_trial アクション

> 実施日時: 2026-03-03
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/update_trial.rs` | 新規 | update_trial アクション |
| `backend/src/domain/actions/trial.rs` | 新規 | trial モジュール定義 |
| `backend/src/domain/actions.rs` | 修正 | `pub mod trial;` を追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo clippy -- -D warnings` エラーなし）

### テスト

成功（32 tests passed）

新規追加のテスト（`domain::actions::trial::update_trial::tests`）:
- `test_update_name` - Trial の名前を更新できる
- `test_update_memo` - Trial のメモを更新できる
- `test_update_both_name_and_memo` - 名前とメモを同時に更新できる
- `test_updated_at_is_changed` - updated_at が更新される
- `test_returns_error_when_trial_completed` - 完了済み Trial の更新は AlreadyCompleted エラー

## コミット情報

- ハッシュ: 8f1b6ac
- ブランチ: task/20260228-trial_model_03.2-domain-action-update-trial

## 次タスクへの申し送り

- `update_trial::Command` は `name: Option<String>`, `memo: Option<String>` の 2 フィールド
- `update_trial::Error::AlreadyCompleted` のみ定義
- `validate` / `execute` / `run` の 3 関数で構成
- `execute` では `Trial::from_raw(...)` を使って updated_at を `Utc::now()` に更新した新インスタンスを返す
- `name` / `memo` が `None` の場合は既存値を維持（no-op ではなく updated_at は更新される）
- 参照: `crate::domain::actions::trial::update_trial`

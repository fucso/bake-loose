# Task Report: update_trial ユースケース

> 実施日時: 2026-03-03
> 依存タスク: 03.2-domain-action-update-trial, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/update_trial.rs` | 新規 | update_trial ユースケース（Input, Error, execute 関数、テスト4件） |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール定義 |
| `backend/src/use_case.rs` | 修正 | `pub mod trial;` を追加 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository を stub から本格的なモック実装に差し替え、MockUnitOfWork に trials フィールドと with_trials コンストラクタを追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo clippy -- -D warnings` ともにエラーなし）

### テスト

成功（85 tests passed、8 integration tests passed）

新規追加のテスト（`use_case::trial::update_trial::tests`）:
- `test_update_trial_name` - Trial の名前を更新できる
- `test_update_trial_memo` - Trial のメモを更新できる
- `test_returns_error_when_trial_not_found` - Trial が存在しない場合 TrialNotFound エラー
- `test_returns_domain_error_when_completed` - 完了済み Trial の更新は Domain エラー

## コミット情報

- ハッシュ: 329ceb2
- ブランチ: task/20260228-trial_model_06.2-use-case-update-trial

## 次タスクへの申し送り

- `update_trial::Input` は `trial_id: TrialId`, `name: Option<String>`, `memo: Option<String>` の3フィールド
- `update_trial::Error` は `Domain(update_trial::Error)`, `TrialNotFound`, `Infrastructure(String)` の3バリアント
- `execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error>` で呼び出し
- `MockTrialRepository` は `Arc<Mutex<Vec<Trial>>>` ベースの本格的なモック実装に差し替え済み
  - `find_by_id`, `find_by_project_id`, `save`, `delete` が全て動作する
- `MockUnitOfWork::with_trials(trials: Vec<Trial>)` コンストラクタを追加済み（Trial を事前登録したい場合に使用）
- 参照: `crate::use_case::trial::update_trial`

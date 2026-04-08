# Task Report: complete_step ユースケース

> 実施日時: 2026-03-03
> 依存タスク: 03.6-domain-action-complete-step, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/complete_step.rs` | 新規 | complete_step ユースケース（Input, Error, execute 関数、テスト5件） |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール宣言 |
| `backend/src/use_case.rs` | 修正 | trial モジュール追加 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository を todo! stub から実装に差し替え、MockUnitOfWork に trials フィールドと with_trials コンストラクタを追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo clippy -- -D warnings` エラーなし、`cargo fmt -- --check` エラーなし）

### テスト

成功（86 tests passed）

新規追加のテスト:
- `test_complete_step_success` - Step を完了できる
- `test_complete_step_with_specified_time` - 指定した日時で完了できる
- `test_returns_error_when_trial_not_found` - Trial が存在しない場合 TrialNotFound エラー
- `test_returns_domain_error_when_step_not_found` - Step が存在しない場合 Domain エラー
- `test_returns_domain_error_when_already_completed` - 既に完了済みの場合 Domain エラー

## コミット情報

- ハッシュ: dd757c1
- ブランチ: task/20260228-trial_model_06.6-use-case-complete-step

## 次タスクへの申し送り

- `complete_step::Input` は `trial_id: TrialId`, `step_id: StepId`, `completed_at: Option<DateTime<Utc>>` を持つ
- `complete_step::Error` は `Domain(complete_step::Error)` / `TrialNotFound` / `Infrastructure(String)` の 3 バリアント
- `complete_step::execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error>` で実行
- `MockTrialRepository` を todo! stub から `Arc<Mutex<Vec<Trial>>>` ベースの実装に差し替え済み
- `MockUnitOfWork::with_trials(trials: Vec<Trial>)` コンストラクタを追加済み（他の trial 系ユースケーステストでも利用可能）
- ユースケースは `crate::use_case::trial::complete_step` に配置

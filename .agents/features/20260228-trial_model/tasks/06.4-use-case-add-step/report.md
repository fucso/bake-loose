# Task Report: add_step ユースケース

> 実施日時: 2026-03-03
> 依存タスク: 03.4-domain-action-add-step, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/add_step.rs` | 新規 | add_step ユースケース（Input, Error, execute 関数、テスト4件） |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール |
| `backend/src/use_case.rs` | 修正 | `pub mod trial;` 追加 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository を stub から本格実装に差し替え、MockUnitOfWork に trials フィールドと with_trials() を追加 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo fmt -- --check`、`cargo clippy -- -D warnings` ともにエラーなし）

### テスト

成功（85 unit tests + 8 integration tests passed）

新規追加のテスト（add_step ユースケース関連 4件）:

正常系:
- `test_add_step_success` - Step を追加でき、リポジトリに保存される
- `test_add_step_with_parameters` - Parameters を含む Step を追加できる

異常系:
- `test_returns_error_when_trial_not_found` - Trial が存在しない場合 TrialNotFound エラー
- `test_returns_domain_error_when_trial_completed` - 完了済み Trial への追加は Domain エラー

## コミット情報

- ハッシュ: fe5e75f
- ブランチ: task/20260228-trial_model_06.4-use-case-add-step

## 次タスクへの申し送り

- `use_case::trial::add_step::execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error>` が実装済み
- `Input` は `trial_id: TrialId`、`name: String`、`started_at: Option<DateTime<Utc>>`、`parameters: Vec<ParameterInput>` を持つ
- `ParameterInput` は `content: ParameterContent` を持つ
- `Error` は `Domain(add_step::Error)`、`TrialNotFound`、`Infrastructure(String)` の3バリアント
- ロジック: トランザクション開始 → find_by_id → ドメインアクション実行 → save → コミット
- `MockTrialRepository` が本格実装済み（find_by_id, find_by_project_id, save, delete すべて動作する）
- `MockUnitOfWork::with_trials(trials)` でテスト用に Trial を事前セット可能
- モジュールパス: `crate::use_case::trial::add_step`

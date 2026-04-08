# Task Report: complete_trial ユースケース

> 実施日時: 2026-03-03
> 依存タスク: 03.3-domain-action-complete-trial, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/complete_trial.rs` | 新規 | complete_trial ユースケース（Input, Error, execute + テスト） |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール定義 |
| `backend/src/use_case.rs` | 修正 | `pub mod trial;` を追加 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository を stub から機能的な実装に差し替え |

### 設計詳細

- **Input**: `trial_id: TrialId`
- **Error**: `Domain(complete_trial::Error)`, `TrialNotFound`, `Infrastructure(String)`
- **ロジック**: トランザクション開始 → find_by_id → complete_trial::run → save → commit
- MockTrialRepository を `Arc<Mutex<Vec<Trial>>>` パターンで実装し、MockProjectRepository と同様の構造に統一

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo clippy -- -D warnings` でエラー・警告なし）

### テスト

成功（84 unit tests + 8 integration tests passed）

新規追加のテスト:
- `test_complete_trial_success` - Trial を完了できる
- `test_returns_error_when_trial_not_found` - Trial が存在しない場合 TrialNotFound エラー
- `test_returns_domain_error_when_already_completed` - 既に完了済みの場合 Domain エラー

## コミット情報

- ハッシュ: 776c9fe
- ブランチ: task/20260228-trial_model_06.3-use-case-complete-trial

## 次タスクへの申し送り

- `use_case::trial::complete_trial::execute<U: UnitOfWork>(uow, input) -> Result<Trial, Error>` が実装済み
- `Input { trial_id: TrialId }` で呼び出し
- エラー型: `Error::Domain(complete_trial::Error)`, `Error::TrialNotFound`, `Error::Infrastructure(String)`
- MockTrialRepository が機能的な実装に更新済み（find_by_id, find_by_project_id, save, delete が動作する）
- 他の trial ユースケース（06.1, 06.2, 06.4〜06.6）でも MockTrialRepository をそのまま利用可能
- モジュール参照: `crate::use_case::trial::complete_trial`

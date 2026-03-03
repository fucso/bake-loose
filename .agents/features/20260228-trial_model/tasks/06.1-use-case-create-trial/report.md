# Task Report: create_trial ユースケース

> 実施日時: 2026-03-03
> 依存タスク: 03.1-domain-action-create-trial, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/create_trial.rs` | 新規 | create_trial ユースケース（Input, Error, execute, ユニットテスト4件） |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール |
| `backend/src/use_case.rs` | 修正 | trial モジュール追加 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository を stub から機能的なモックに差し替え |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo clippy -- -D warnings`、`cargo fmt -- --check` ともにエラーなし）

### テスト

成功（85 unit tests + 8 integration tests passed）

新規追加のテスト:
- `test_create_trial_success` - Trial を作成できる
- `test_create_trial_with_steps` - Steps を含む Trial を作成できる
- `test_returns_error_when_project_not_found` - プロジェクトが存在しない場合 ProjectNotFound エラー
- `test_returns_domain_error` - ドメインエラーが正しく伝播される

## コミット情報

- ハッシュ: 0c1d8c3
- ブランチ: task/20260228-trial_model_06.1-use-case-create-trial

## 次タスクへの申し送り

- `create_trial::Input` は `project_id: ProjectId`, `name: Option<String>`, `memo: Option<String>`, `steps: Vec<create_trial::StepInput>` を受け取る
- `create_trial::execute(uow, input)` で Trial を作成。プロジェクト存在確認 → ドメインアクション → 永続化の順
- `create_trial::Error` は `Domain(create_trial::Error)` / `ProjectNotFound` / `Infrastructure(String)` の 3 バリアント
- `MockTrialRepository` が機能的なモックに差し替え済み（`Arc<Mutex<Vec<Trial>>>` 使用）
- `MockUnitOfWork` に `trials` フィールドが追加済み
- モジュールパス: `crate::use_case::trial::create_trial`

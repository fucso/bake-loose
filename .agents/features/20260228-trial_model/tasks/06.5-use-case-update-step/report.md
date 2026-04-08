# Task Report: update_step ユースケース

> 実施日時: 2026-03-03
> 依存タスク: 03.5-domain-action-update-step, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/update_step.rs` | 新規 | update_step ユースケース実装 |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール宣言 |
| `backend/src/use_case.rs` | 修正 | trial モジュールを追加 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository を stub から本格実装に差し替え、MockUnitOfWork に trials フィールドと with_trials コンストラクタを追加 |

## 実装詳細

### 型定義

- `Input`: ユースケース入力（`trial_id`, `step_id`, `name`, `started_at`, `add_parameters`, `remove_parameter_ids`）
- `ParameterInput`: パラメーター入力（`content: ParameterContent`）
- `Error`: ユースケースエラー（`Domain(update_step::Error)`, `TrialNotFound`, `Infrastructure(String)`）

### ロジック

1. `uow.begin()` でトランザクション開始
2. `trial_repository.find_by_id()` で Trial を取得
3. 存在しない場合は rollback して `TrialNotFound` エラー
4. `update_step::run()` ドメインアクションを実行
5. ドメインエラーの場合は rollback して `Domain` エラー
6. `trial_repository.save()` で永続化
7. `uow.commit()` でコミット
8. 更新後の Trial を返す

### MockTrialRepository の本格実装

- `todo!()` マクロによる stub から `Arc<Mutex<Vec<Trial>>>` を使った本格的なモックに差し替え
- `find_by_id`, `find_by_project_id`, `save`, `delete` の全メソッドを実装
- `MockUnitOfWork` に `trials` フィールドと `with_trials(trials: Vec<Trial>)` コンストラクタを追加

## ビルド・テスト結果

### コンパイル/ビルド

- `cargo check`: 成功
- `cargo clippy -- -D warnings`: 警告なし
- `cargo fmt`: 適用済み

### テスト

成功（86 tests passed、8 integration tests passed）

| テスト名 | 結果 |
|----------|------|
| `test_update_step_name` | ok |
| `test_add_parameters` | ok |
| `test_remove_parameters` | ok |
| `test_returns_error_when_trial_not_found` | ok |
| `test_returns_domain_error_when_step_not_found` | ok |

## コミット情報

- ハッシュ: 180cd33
- ブランチ: task/20260228-trial_model_06.5-use-case-update-step

## 次タスクへの申し送り

- `update_step` ユースケースは `crate::use_case::trial::update_step` モジュールに配置
- `execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error>` がエントリポイント
- `MockTrialRepository` は `todo!()` stub から本格実装に差し替え済み。他の trial ユースケースタスク（06.1〜06.4, 06.6）でも `MockUnitOfWork::with_trials(vec![...])` を使用可能
- `use_case/trial.rs` モジュールファイルを新規作成済み。他の trial ユースケースはここにモジュール宣言を追加すること

# Task Report: update_step アクション

> 実施日時: 2026-03-03
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/update_step.rs` | 新規 | update_step アクション実装 |
| `backend/src/domain/actions/trial.rs` | 新規 | trial アクションモジュール宣言 |
| `backend/src/domain/actions.rs` | 修正 | trial モジュールを追加 |

## 実装詳細

### 型定義

- `ParameterValidationError`: パラメーター検証エラー (`EmptyKey`, `EmptyText`)
- `ParameterInput`: 追加パラメーターの入力型 (`content: ParameterContent`)
- `Command`: アクションコマンド (`step_id`, `name`, `started_at`, `add_parameters`, `remove_parameter_ids`)
- `Error`: アクションエラー (`TrialAlreadyCompleted`, `StepNotFound`, `StepAlreadyCompleted`, `EmptyStepName`, `InvalidParameter`, `ParameterNotFound`)

### バリデーション (validate)

1. Trial のステータスが Completed の場合はエラー
2. 指定 Step が Trial 内に存在するか検証
3. Step の `completed_at` が Some の場合はエラー
4. name が `Some("")` の場合はエラー
5. `add_parameters` の各 content を検証（空 key / 空 text）
6. `remove_parameter_ids` が既存パラメーターに存在するか検証

### 状態遷移 (execute)

- Trial の全フィールドを getters 経由で取得し `Trial::from_raw` で再構築
- 対象 Step を見つけ name / started_at を更新
- `remove_parameter_ids` でフィルタリング後 `add_parameters` を追加
- Step と Trial の `updated_at` を `Utc::now()` で更新

### 注意点

- `started_at: Option<Option<DateTime<Utc>>>` の3値区別（None=変更なし, Some(None)=クリア, Some(Some(v))=更新）
- `DateTime<Utc>` は `Copy` のため match 内で `.copied()` / デリファレンスを活用

## ビルド・テスト結果

### コンパイル/ビルド

- `cargo check`: 成功
- `cargo clippy -- -D warnings`: 警告なし

> 注: `cargo build` はリンク時の OOM (ld が signal 9 で終了) により失敗。
> これはコンテナのメモリ制限によるインフラの問題であり、コードの問題ではない。
> `cargo check` および `cargo test` は正常に完了している。

### テスト

成功（14 tests passed）

| テスト名 | 結果 |
|----------|------|
| `test_update_step_name` | ok |
| `test_update_step_started_at` | ok |
| `test_clear_step_started_at` | ok |
| `test_add_parameters` | ok |
| `test_remove_parameters` | ok |
| `test_add_and_remove_parameters` | ok |
| `test_step_updated_at_is_changed` | ok |
| `test_trial_updated_at_is_changed` | ok |
| `test_returns_error_when_trial_completed` | ok |
| `test_returns_error_when_step_not_found` | ok |
| `test_returns_error_when_step_completed` | ok |
| `test_returns_error_when_name_empty` | ok |
| `test_returns_error_when_parameter_invalid` | ok |
| `test_returns_error_when_parameter_not_found` | ok |

## コミット情報

- ハッシュ: 5eb2197
- ブランチ: task/20260228-trial_model_03.5-domain-action-update-step

## 次タスクへの申し送り

- `update_step::Command` は `step_id`, `name: Option<String>`, `started_at: Option<Option<DateTime<Utc>>>`, `add_parameters: Vec<ParameterInput>`, `remove_parameter_ids: Vec<ParameterId>` を持つ
- `update_step::ParameterInput` は `content: ParameterContent` のみを持つシンプルな型
- `update_step::ParameterValidationError` は `EmptyKey`, `EmptyText` の 2 バリアント
- `update_step::run(state: Trial, command: Command) -> Result<Trial, Error>` がメインエントリポイント
- モジュールパス: `crate::domain::actions::trial::update_step`

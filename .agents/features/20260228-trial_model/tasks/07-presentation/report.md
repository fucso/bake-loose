# Task Report: GraphQL 実装

> 実施日時: 2026-03-03
> 依存タスク: 05-repository, 06.1〜06.6

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/Cargo.toml` | 修正 | async-graphql に chrono feature 追加 |
| `backend/src/presentation/graphql/types/trial.rs` | 新規 | Trial, Step, Parameter GraphQL 型、入力型定義 |
| `backend/src/presentation/graphql/types/parameter_content.rs` | 新規 | ParameterContent union、ParameterValue union、関連型 |
| `backend/src/presentation/graphql/types.rs` | 修正 | trial, parameter_content モジュール追加 |
| `backend/src/presentation/graphql/query/trial.rs` | 新規 | TrialQuery（trial, trialsByProject） |
| `backend/src/presentation/graphql/query.rs` | 修正 | trial モジュール追加 |
| `backend/src/presentation/graphql/mutation/trial.rs` | 新規 | TrialMutation（6つのミューテーション） |
| `backend/src/presentation/graphql/mutation.rs` | 修正 | trial モジュール追加 |
| `backend/src/presentation/graphql/error.rs` | 修正 | Trial 関連8ユースケースのエラー変換追加 |
| `backend/src/presentation/graphql/schema.rs` | 修正 | QueryRoot, MutationRoot に Trial 追加 |
| `backend/src/use_case/trial/get_trial.rs` | 新規 | ID によるトライアル取得ユースケース |
| `backend/src/use_case/trial/list_trials_by_project.rs` | 新規 | プロジェクト別トライアル一覧ユースケース |
| `backend/src/use_case/trial.rs` | 修正 | get_trial, list_trials_by_project モジュール追加 |
| `backend/src/use_case/trial/update_step.rs` | 修正 | テスト内の ParameterValue::Text 構文修正 |
| `backend/tests/graphql.rs` | 修正 | trial テストモジュール追加 |
| `backend/tests/graphql/trial.rs` | 新規 | Trial テストモジュール宣言 |
| `backend/tests/graphql/trial/create_trial.rs` | 新規 | createTrial テスト |
| `backend/tests/graphql/trial/query_trial.rs` | 新規 | trial クエリテスト |
| `backend/tests/graphql/trial/query_trials_by_project.rs` | 新規 | trialsByProject クエリテスト |
| `backend/tests/graphql/trial/add_step.rs` | 新規 | addStep テスト |
| `backend/tests/graphql/trial/update_step.rs` | 新規 | updateStep テスト |
| `backend/tests/graphql/trial/complete_step.rs` | 新規 | completeStep テスト |
| `backend/tests/graphql/trial/complete_trial.rs` | 新規 | completeTrial テスト + 完了済み更新エラーテスト |
| `backend/tests/graphql/trial/update_trial.rs` | 新規 | updateTrial テスト |
| `backend/tests/fixtures/trials.sql` | 新規 | テストフィクスチャ |

## ビルド・テスト結果

### コンパイル/ビルド

成功（cargo build, cargo fmt, cargo clippy -- -D warnings すべてパス）

### テスト

成功（114 ユニットテスト + 20 統合テスト すべてパス）

新規追加した統合テスト:
- `test_create_trial` - Trial を作成できる
- `test_query_trial` - Trial を取得できる
- `test_query_trials_by_project` - プロジェクト別に Trial を取得できる
- `test_add_step` - Step を追加できる
- `test_update_step` - Step を更新できる
- `test_complete_step` - Step を完了できる
- `test_complete_trial` - Trial を完了できる
- `test_create_trial_with_invalid_project` - 存在しないプロジェクトでエラー
- `test_update_completed_trial_returns_error` - 完了済み Trial の更新でエラー
- `test_returns_null_when_not_found` - 存在しない Trial で null
- `test_returns_empty_list` - 空のトライアル一覧

## コミット情報

- ハッシュ: 197043f（実装）, dbd56e1（cargo fmt 修正）
- ブランチ: task/20260228-trial_model_07-presentation

## 次タスクへの申し送り

- `async-graphql` に `chrono` feature を追加（DateTime を InputType/OutputType として使用するため）
- Query 用ユースケース `get_trial` と `list_trials_by_project` を追加作成（タスク分解に含まれていなかったが、presentation → use_case の依存方向を守るために必要）
- `update_step.rs` テスト内の `ParameterValue::Text("...")` を `ParameterValue::Text { value: "..." }` に修正（05-repository タスクで変更された struct variant 形式への対応漏れ）
- GraphQL ParameterInput は oneOf 的な設計（keyValue / duration / timeMarker / text のいずれか1つを指定）。バリデーションはリゾルバー内の `convert_parameter_content` で実施
- エラーコード規約: `NOT_FOUND`, `VALIDATION_ERROR`, `INTERNAL_ERROR` を使用

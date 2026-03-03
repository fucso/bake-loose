# Task Report: PgTrialRepository 実装

> 実施日時: 2026-03-03
> 依存タスク: 01-migration, 04-ports

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/repository/models/trial_row.rs` | 新規 | TrialRow, StepRow, ParameterRow DB モデル |
| `backend/src/repository/trial_repo.rs` | 新規 | PgTrialRepository 実装 + 統合テスト |
| `backend/src/repository/models.rs` | 修正 | trial_row モジュール追加 |
| `backend/src/repository.rs` | 修正 | trial_repo モジュール追加 |
| `backend/src/repository/pg_unit_of_work.rs` | 修正 | PgTrialRepositoryStub を PgTrialRepository に差し替え |
| `backend/src/domain/models/parameter.rs` | 修正 | ParameterValue::Text を struct variant に変更 |
| `backend/src/domain/actions/trial/create_trial.rs` | 修正 | ParameterValue::Text の使用箇所を更新 |
| `backend/src/domain/actions/trial/add_step.rs` | 修正 | ParameterValue::Text の使用箇所を更新 |
| `backend/src/domain/actions/trial/update_step.rs` | 修正 | ParameterValue::Text の使用箇所を更新 |

### 実装詳細

#### DB モデル (Row 構造体)

- `TrialRow`: trials テーブルの行を表す。`into_trial()` メソッドで Steps を受け取りドメインモデルに変換
- `StepRow`: steps テーブルの行を表す。`into_step()` メソッドで Parameters を受け取りドメインモデルに変換
- `ParameterRow`: parameters テーブルの行を表す。`From<ParameterRow> for Parameter` で JSONB からドメインモデルに変換
- `trial_status_to_str()`: TrialStatus から DB 文字列への変換ヘルパー

#### PgTrialRepository

- `find_by_id`: Trial → Steps (position 順) → Parameters の順に取得し、階層構造に組み立て
- `find_by_project_id`: N+1 回避のため `ANY($1)` で Steps/Parameters を一括取得し、HashMap でグループ化
- `save`: Trial を UPSERT → 既存 Steps を DELETE (CASCADE で Parameters も削除) → 新 Steps/Parameters を INSERT
- `delete`: Trial を DELETE (CASCADE で Steps/Parameters も自動削除)

#### ParameterValue::Text の修正 (ドメインモデル変更)

`ParameterValue` に `#[serde(tag = "type")]` (内部タグ付き) が設定されているが、serde は newtype variant (`Text(String)`) を内部タグ付きでシリアライズできない制約がある。JSONB 保存のために `Text(String)` → `Text { value: String }` に変更した。

この変更は 02-domain-model タスクの成果物に対する修正だが、JSONB ストレージの正常動作に必須であるため本タスクで対応した。

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo clippy -- -D warnings` ともにエラーなし）

### テスト

成功（97 tests passed）

- unit tests: 89 passed (うち trial_repo テスト 8 件)
- graphql integration tests: 8 passed
- doc tests: 1 ignored

#### 新規テスト一覧

| テスト名 | 内容 |
|----------|------|
| `test_save_and_find_trial` | Trial を保存して取得できる |
| `test_save_trial_with_steps` | Steps を含む Trial を保存して取得できる |
| `test_save_trial_with_parameters` | Parameters を含む Trial を保存して取得できる |
| `test_find_by_project_id` | プロジェクト別に Trial を取得できる |
| `test_update_trial` | 既存の Trial を更新できる |
| `test_delete_trial` | Trial を削除できる |
| `test_delete_cascades_to_steps` | Trial 削除時に Steps も削除される |
| `test_parameter_content_json_roundtrip` | 全 ParameterContent バリアントの JSON 変換が正しく動作する |

## コミット情報

- ハッシュ: de91e04
- ブランチ: task/20260228-trial_model_05-repository

## 次タスクへの申し送り

- `PgTrialRepository` は `backend/src/repository/trial_repo.rs` に実装済み
- `PgUnitOfWork::trial_repository()` は `PgTrialRepository` を返す（stub は削除済み）
- `ParameterValue::Text` が `Text(String)` から `Text { value: String }` に変更された
  - 06.x ユースケースタスクで `ParameterValue::Text` を使用する場合は struct variant 形式を使用すること
  - 07-presentation タスクで GraphQL 入力型を定義する場合も同様
- save は全置換方式（既存の Steps/Parameters を削除して新規挿入）
- find_by_project_id は N+1 回避のため `ANY($1)` で一括取得
- ParameterContent の JSONB 変換は serde の Serialize/Deserialize を使用

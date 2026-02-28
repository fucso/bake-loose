# Task: add_step アクション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

既存の Trial に新しい Step を追加するドメインアクションを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/add_step.rs` | 新規 | add_step アクション |
| `backend/src/domain/actions/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Command

- `name`: String - ステップ名（必須）
- `started_at`: Option<DateTime<Utc>> - 開始日時
- `parameters`: Vec<ParameterInput> - 初期パラメーター

### ParameterInput

- `content`: ParameterContent - パラメーター内容

### Error

- `TrialAlreadyCompleted` - Trial が既に完了している
- `EmptyStepName` - ステップ名が空
- `InvalidParameter { parameter_index: usize, reason: ParameterValidationError }` - パラメーターが不正

### ロジック

1. Trial のステータスが Completed の場合はエラー
2. Step 名が空でないことを検証
3. Parameter を検証（create_trial と同じルール）
4. 新しい Step の position を既存の Steps の最大値 + 1 に設定
5. Step を Trial に追加
6. updated_at を現在時刻に更新
7. 更新後の Trial を返す

### 注意点

- position は既存 Steps の最大値 + 1（Steps が空の場合は 0）
- Trial の updated_at も更新される

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/add_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_add_step_to_empty_trial` | Steps が空の Trial に Step を追加できる |
| `test_add_step_position_is_zero_for_first` | 最初の Step の position は 0 |
| `test_add_step_position_increments` | 追加した Step の position が正しくインクリメントされる |
| `test_add_step_with_parameters` | Parameter を含む Step を追加できる |
| `test_add_step_with_started_at` | started_at を設定して追加できる |
| `test_trial_updated_at_is_changed` | Trial の updated_at が更新される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_completed` | 完了済み Trial への追加は TrialAlreadyCompleted エラー |
| `test_returns_error_when_step_name_empty` | Step 名が空の場合 EmptyStepName エラー |
| `test_returns_error_when_parameter_invalid` | 不正なパラメーターの場合 InvalidParameter エラー |

---

## 完了条件

- [ ] Command, Error が定義されている
- [ ] validate / execute / run 関数が実装されている
- [ ] position が正しく計算される
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

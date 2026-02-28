# Task: complete_step アクション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

Step を完了状態にし、完了日時を記録するドメインアクションを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/complete_step.rs` | 新規 | complete_step アクション |
| `backend/src/domain/actions/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Command

- `step_id`: StepId - 完了対象の Step ID
- `completed_at`: Option<DateTime<Utc>> - 完了日時（None の場合は現在時刻）

### Error

- `TrialAlreadyCompleted` - Trial が既に完了している
- `StepNotFound` - 指定された Step が存在しない
- `StepAlreadyCompleted` - Step が既に完了している

### ロジック

1. Trial のステータスが Completed の場合はエラー
2. 指定された Step が Trial 内に存在するか検証
3. Step が既に Completed の場合はエラー
4. Step の completed_at を設定（Command で指定がなければ現在時刻）
5. Step と Trial の updated_at を更新
6. 更新後の Trial を返す

### 注意点

- 完了日時は明示的に指定可能（過去の記録をつける場合など）
- 完了済み Step の再完了は許可しない

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/complete_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_complete_step` | Step を完了できる |
| `test_completed_at_is_set` | completed_at が設定される |
| `test_complete_step_with_specified_time` | 指定した日時で完了できる |
| `test_step_updated_at_is_changed` | Step の updated_at が更新される |
| `test_trial_updated_at_is_changed` | Trial の updated_at が更新される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_completed` | 完了済み Trial の場合 TrialAlreadyCompleted エラー |
| `test_returns_error_when_step_not_found` | Step が存在しない場合 StepNotFound エラー |
| `test_returns_error_when_step_already_completed` | 既に完了済みの場合 StepAlreadyCompleted エラー |

---

## 完了条件

- [ ] Command, Error が定義されている
- [ ] validate / execute / run 関数が実装されている
- [ ] completed_at が正しく設定される
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

# Task: complete_trial アクション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

Trial を完了ステータスに変更するドメインアクションを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/complete_trial.rs` | 新規 | complete_trial アクション |
| `backend/src/domain/actions/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Command

- なし（Trial の状態のみで判断）

### Error

- `AlreadyCompleted` - Trial が既に完了している

### ロジック

1. Trial のステータスが Completed の場合はエラー
2. ステータスを Completed に変更
3. updated_at を現在時刻に更新
4. 更新後の Trial を返す

### 注意点

- 未完了の Step があっても Trial を完了できる（ビジネス判断による）
- 完了後は update_trial も不可になる

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/complete_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_complete_trial` | InProgress の Trial を完了できる |
| `test_status_is_completed` | 完了後のステータスが Completed |
| `test_updated_at_is_changed` | updated_at が更新される |
| `test_complete_trial_with_incomplete_steps` | 未完了の Step があっても完了できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_already_completed` | 既に完了済みの場合 AlreadyCompleted エラー |

---

## 完了条件

- [ ] Command（空）, Error が定義されている
- [ ] validate / execute / run 関数が実装されている
- [ ] 完了済み Trial の再完了がエラーになる
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

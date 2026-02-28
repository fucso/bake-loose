# Task: update_trial アクション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

既存の Trial の名前・メモを更新するドメインアクションを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/actions/trial/update_trial.rs` | 新規 | update_trial アクション |
| `backend/src/domain/actions/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Command

- `name`: Option<String> - 新しい名前（None の場合は変更なし）
- `memo`: Option<String> - 新しいメモ（None の場合は変更なし）

### Error

- `AlreadyCompleted` - Trial が既に完了している場合は更新不可

### ロジック

1. Trial のステータスが Completed の場合はエラー
2. name が Some の場合は更新
3. memo が Some の場合は更新
4. updated_at を現在時刻に更新
5. 更新後の Trial を返す

### 注意点

- name と memo の両方が None の場合でも、updated_at は更新される（no-op ではない）
- 完了済み Trial の更新は許可しない

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/domain/actions/trial/update_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_update_name` | Trial の名前を更新できる |
| `test_update_memo` | Trial のメモを更新できる |
| `test_update_both_name_and_memo` | 名前とメモを同時に更新できる |
| `test_updated_at_is_changed` | updated_at が更新される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_completed` | 完了済み Trial の更新は AlreadyCompleted エラー |

---

## 完了条件

- [ ] Command, Error が定義されている
- [ ] validate / execute / run 関数が実装されている
- [ ] 完了済み Trial の更新がエラーになる
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

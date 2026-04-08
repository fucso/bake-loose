# Task: update_trial ユースケース

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 03.2-domain-action-update-trial, 04-ports

## 目的

Trial の名前・メモを更新するユースケースを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/update_trial.rs` | 新規 | update_trial ユースケース |
| `backend/src/use_case/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Input

- `trial_id`: TrialId - 更新対象の Trial ID
- `name`: Option<String> - 新しい名前
- `memo`: Option<String> - 新しいメモ

### Error

- `Domain(update_trial::Error)` - ドメインアクションのエラー
- `TrialNotFound` - 指定された Trial が存在しない
- `Infrastructure(String)` - 永続化エラー

### ロジック

1. トランザクション開始
2. trial_repository.find_by_id で Trial を取得
3. 存在しない場合は TrialNotFound エラー
4. update_trial ドメインアクションを実行
5. trial_repository.save で永続化
6. コミット
7. 更新後の Trial を返す

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/use_case/trial/update_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_update_trial_name` | Trial の名前を更新できる |
| `test_update_trial_memo` | Trial のメモを更新できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_not_found` | Trial が存在しない場合 TrialNotFound エラー |
| `test_returns_domain_error_when_completed` | 完了済み Trial の更新は Domain エラー |

---

## 完了条件

- [ ] Input, Error が定義されている
- [ ] execute 関数が実装されている
- [ ] Trial の存在確認を行っている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

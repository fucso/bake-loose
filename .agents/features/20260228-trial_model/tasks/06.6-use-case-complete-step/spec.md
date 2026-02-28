# Task: complete_step ユースケース

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 03.6-domain-action-complete-step, 04-ports

## 目的

Step を完了状態にし、完了日時を記録するユースケースを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/complete_step.rs` | 新規 | complete_step ユースケース |
| `backend/src/use_case/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Input

- `trial_id`: TrialId - 所属する Trial ID
- `step_id`: StepId - 完了対象の Step ID
- `completed_at`: Option<DateTime<Utc>> - 完了日時（None の場合は現在時刻）

### Error

- `Domain(complete_step::Error)` - ドメインアクションのエラー
- `TrialNotFound` - 指定された Trial が存在しない
- `Infrastructure(String)` - 永続化エラー

### ロジック

1. トランザクション開始
2. trial_repository.find_by_id で Trial を取得
3. 存在しない場合は TrialNotFound エラー
4. complete_step ドメインアクションを実行
5. trial_repository.save で永続化
6. コミット
7. 更新後の Trial を返す

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/use_case/trial/complete_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_complete_step_success` | Step を完了できる |
| `test_complete_step_with_specified_time` | 指定した日時で完了できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_not_found` | Trial が存在しない場合 TrialNotFound エラー |
| `test_returns_domain_error_when_step_not_found` | Step が存在しない場合 Domain エラー |
| `test_returns_domain_error_when_already_completed` | 既に完了済みの場合 Domain エラー |

---

## 完了条件

- [ ] Input, Error が定義されている
- [ ] execute 関数が実装されている
- [ ] Trial の存在確認を行っている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

# Task: add_step ユースケース

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 03.4-domain-action-add-step, 04-ports

## 目的

既存の Trial に新しい Step を追加するユースケースを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/add_step.rs` | 新規 | add_step ユースケース |
| `backend/src/use_case/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Input

- `trial_id`: TrialId - 追加先の Trial ID
- `name`: String - ステップ名
- `started_at`: Option<DateTime<Utc>> - 開始日時
- `parameters`: Vec<ParameterInput> - 初期パラメーター

### Error

- `Domain(add_step::Error)` - ドメインアクションのエラー
- `TrialNotFound` - 指定された Trial が存在しない
- `Infrastructure(String)` - 永続化エラー

### ロジック

1. トランザクション開始
2. trial_repository.find_by_id で Trial を取得
3. 存在しない場合は TrialNotFound エラー
4. add_step ドメインアクションを実行
5. trial_repository.save で永続化
6. コミット
7. 更新後の Trial を返す

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/use_case/trial/add_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_add_step_success` | Step を追加できる |
| `test_add_step_with_parameters` | Parameters を含む Step を追加できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_not_found` | Trial が存在しない場合 TrialNotFound エラー |
| `test_returns_domain_error_when_trial_completed` | 完了済み Trial への追加は Domain エラー |

---

## 完了条件

- [ ] Input, Error が定義されている
- [ ] execute 関数が実装されている
- [ ] Trial の存在確認を行っている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

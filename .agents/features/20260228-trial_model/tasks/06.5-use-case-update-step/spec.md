# Task: update_step ユースケース

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 03.5-domain-action-update-step, 04-ports

## 目的

既存の Step の名前・開始日時・パラメーターを更新するユースケースを実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/update_step.rs` | 新規 | update_step ユースケース |
| `backend/src/use_case/trial.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Input

- `trial_id`: TrialId - 所属する Trial ID
- `step_id`: StepId - 更新対象の Step ID
- `name`: Option<String> - 新しい名前
- `started_at`: Option<Option<DateTime<Utc>>> - 新しい開始日時
- `add_parameters`: Vec<ParameterInput> - 追加するパラメーター
- `remove_parameter_ids`: Vec<ParameterId> - 削除するパラメーター ID

### Error

- `Domain(update_step::Error)` - ドメインアクションのエラー
- `TrialNotFound` - 指定された Trial が存在しない
- `Infrastructure(String)` - 永続化エラー

### ロジック

1. トランザクション開始
2. trial_repository.find_by_id で Trial を取得
3. 存在しない場合は TrialNotFound エラー
4. update_step ドメインアクションを実行
5. trial_repository.save で永続化
6. コミット
7. 更新後の Trial を返す

### 注意点

- Step の存在確認はドメインアクションで行う
- Trial を取得して Step を更新し、Trial として保存する（aggregate root 経由）

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/use_case/trial/update_step.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_update_step_name` | Step の名前を更新できる |
| `test_add_parameters` | パラメーターを追加できる |
| `test_remove_parameters` | パラメーターを削除できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_trial_not_found` | Trial が存在しない場合 TrialNotFound エラー |
| `test_returns_domain_error_when_step_not_found` | Step が存在しない場合 Domain エラー |

---

## 完了条件

- [ ] Input, Error が定義されている
- [ ] execute 関数が実装されている
- [ ] Trial の存在確認を行っている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

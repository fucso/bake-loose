# Task: create_trial ユースケース

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 03.1-domain-action-create-trial, 04-ports

## 目的

Trial を作成するユースケースを実装する。プロジェクトの存在確認を行い、ドメインアクションを呼び出して永続化する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case/trial/create_trial.rs` | 新規 | create_trial ユースケース |
| `backend/src/use_case/trial.rs` | 新規 | trial ユースケースモジュール |
| `backend/src/use_case.rs` | 修正 | trial モジュール追加 |

---

## 設計詳細

### Input

- `project_id`: ProjectId - 所属プロジェクト
- `name`: Option<String> - 任意の名前
- `memo`: Option<String> - 備考・ノート
- `steps`: Vec<StepInput> - 初期ステップ

### Error

- `Domain(create_trial::Error)` - ドメインアクションのエラー
- `ProjectNotFound` - 指定されたプロジェクトが存在しない
- `Infrastructure(String)` - 永続化エラー

### ロジック

1. トランザクション開始
2. project_repository で project_id の存在確認
3. 存在しない場合は ProjectNotFound エラー
4. create_trial ドメインアクションを実行
5. trial_repository.save で永続化
6. コミット
7. 作成した Trial を返す

---

## テストケース

### テストファイル

- **ユニットテスト**: `backend/src/use_case/trial/create_trial.rs` 内の `#[cfg(test)] mod tests`（MockUnitOfWork 使用）

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_success` | Trial を作成できる |
| `test_create_trial_with_steps` | Steps を含む Trial を作成できる |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_project_not_found` | プロジェクトが存在しない場合 ProjectNotFound エラー |
| `test_returns_domain_error` | ドメインエラーが正しく伝播される |

---

## 完了条件

- [ ] Input, Error が定義されている
- [ ] execute 関数が実装されている
- [ ] プロジェクトの存在確認を行っている
- [ ] UnitOfWork 経由で永続化している
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る

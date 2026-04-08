# Task: TrialRepository トレイト

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 02-domain-model

## 目的

Trial の永続化に必要なリポジトリトレイトを定義する。Trial を aggregate root として、Step と Parameter を含めて操作する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/ports/trial_repository.rs` | 新規 | TrialRepository トレイト |
| `backend/src/ports/unit_of_work.rs` | 修正 | TrialRepo 追加 |
| `backend/src/ports.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### TrialRepository トレイト

以下のメソッドを定義:

- `find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError>`
  - Trial とその Steps、Parameters を含めて取得
- `find_by_project_id(&self, project_id: &ProjectId, sort: TrialSort) -> Result<Vec<Trial>, RepositoryError>`
  - プロジェクトに紐づく Trial 一覧を取得
- `save(&self, trial: &Trial) -> Result<(), RepositoryError>`
  - Trial とその Steps、Parameters を保存（UPSERT）
- `delete(&self, id: &TrialId) -> Result<(), RepositoryError>`
  - Trial を削除（CASCADE で Steps、Parameters も削除）

### TrialSortColumn (enum)

- `CreatedAt` - 作成日時
- `UpdatedAt` - 更新日時

### 注意点

- Trial は aggregate root として Steps と Parameters を含む
- save は UPSERT で新規作成・更新の両方に対応
- find_by_id は関連する Steps と Parameters も取得して Trial に含める
- Step や Parameter の個別取得メソッドは提供しない（aggregate root 経由でアクセス）

### UnitOfWork への追加

- `trial_repository(&mut self) -> Self::TrialRepo` を追加

---

## 完了条件

- [ ] TrialRepository トレイトが定義されている
- [ ] TrialSortColumn が定義されている
- [ ] UnitOfWork に trial_repository が追加されている
- [ ] ports.rs にモジュールが追加されている
- [ ] すべてのメソッドが Result 型を返す
- [ ] async_trait + Send + Sync バウンドが設定されている

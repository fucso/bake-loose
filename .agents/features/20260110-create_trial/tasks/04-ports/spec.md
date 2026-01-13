# Task: Ports 定義

> Feature: [create_trial](../../spec.md)
> 依存: [02-domain-models](../02-domain-models/)

## 目的

TrialRepository トレイトを定義し、UnitOfWork に追加する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/ports.rs` | 修正 | trial_repository モジュールの追加 |
| `src/ports/trial_repository.rs` | 新規 | TrialRepository トレイト |
| `src/ports/unit_of_work.rs` | 修正 | TrialRepo の追加 |

---

## 設計詳細

### TrialRepository トレイト

Trial と関連する Step, Parameter の永続化を担当する。

```rust
// src/ports/trial_repository.rs

use crate::domain::models::{
    Parameter, ParameterId, Project, ProjectId, Step, StepId, Trial, TrialId,
};
use crate::ports::error::RepositoryError;

#[async_trait::async_trait]
pub trait TrialRepository: Send + Sync {
    /// Trial を ID で取得
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError>;

    /// Project に紐づく Trial を全件取得
    async fn find_by_project_id(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Trial>, RepositoryError>;

    /// Trial に紐づく Step を全件取得（position 順）
    async fn find_steps_by_trial_id(
        &self,
        trial_id: &TrialId,
    ) -> Result<Vec<Step>, RepositoryError>;

    /// Step に紐づく Parameter を全件取得（position 順）
    async fn find_parameters_by_step_id(
        &self,
        step_id: &StepId,
    ) -> Result<Vec<Parameter>, RepositoryError>;

    /// 複数の Step の Parameter を一括取得（step_id でグループ化）
    async fn find_parameters_by_step_ids(
        &self,
        step_ids: &[StepId],
    ) -> Result<Vec<Parameter>, RepositoryError>;

    /// Trial を保存（UPSERT）
    async fn save_trial(&self, trial: &Trial) -> Result<(), RepositoryError>;

    /// Step を保存（UPSERT）
    async fn save_step(&self, step: &Step) -> Result<(), RepositoryError>;

    /// Parameter を保存（UPSERT）
    async fn save_parameter(&self, parameter: &Parameter) -> Result<(), RepositoryError>;

    /// Trial, Steps, Parameters を一括保存
    async fn save_all(
        &self,
        trial: &Trial,
        steps: &[Step],
        parameters: &[Parameter],
    ) -> Result<(), RepositoryError>;
}
```

**設計ポイント:**

- `save_all` は Trial 作成時に Trial + Steps + Parameters を一括保存するための便利メソッド
- `find_parameters_by_step_ids` は N+1 問題を避けるためのバッチ取得メソッド
- 個別の `save_*` メソッドは将来の部分更新に備えて用意

### UnitOfWork への追加

```rust
// src/ports/unit_of_work.rs

use crate::ports::trial_repository::TrialRepository;

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type ProjectRepo: ProjectRepository;
    type TrialRepo: TrialRepository;  // 追加

    fn project_repository(&mut self) -> Self::ProjectRepo;
    fn trial_repository(&mut self) -> Self::TrialRepo;  // 追加

    async fn begin(&mut self) -> Result<(), RepositoryError>;
    async fn commit(&mut self) -> Result<(), RepositoryError>;
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}
```

---

## テストケース

ports 層はトレイト定義のみのため、ユニットテストは不要。
実装は repository 層でテストする。

---

## 完了条件

- [ ] TrialRepository トレイトが定義されている
- [ ] UnitOfWork に TrialRepo が追加されている
- [ ] 既存の MockUnitOfWork がコンパイルエラーにならないよう対応（スタブ実装）

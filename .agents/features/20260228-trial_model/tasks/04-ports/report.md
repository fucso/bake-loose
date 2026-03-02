# Task Report: TrialRepository トレイト

> 実施日時: 2026-03-03
> 依存タスク: 02-domain-model

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/ports/trial_repository.rs` | 新規 | TrialRepository トレイト、TrialSortColumn、TrialSort の定義 |
| `backend/src/ports/unit_of_work.rs` | 修正 | TrialRepo 関連型と trial_repository() メソッドを追加 |
| `backend/src/ports.rs` | 修正 | trial_repository モジュールと pub use を追加 |
| `backend/src/repository/pg_unit_of_work.rs` | 修正 | PgTrialRepositoryStub（仮実装）を追加して UnitOfWork を実装 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | 修正 | MockTrialRepository stub を追加して MockUnitOfWork を実装 |

## ビルド・テスト結果

### コンパイル/ビルド

成功（`cargo build`、`cargo clippy -- -D warnings` ともにエラーなし）

### テスト

成功（36 tests passed）

- unit tests: 27 passed
- graphql integration tests: 8 passed
- doc tests: 1 ignored

## コミット情報

- ハッシュ: 0c075fb
- ブランチ: task/20260228-trial_model_04-ports

## 次タスクへの申し送り

- `TrialRepository` トレイトは `backend/src/ports/trial_repository.rs` に定義済み
- メソッド一覧:
  - `find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError>`
  - `find_by_project_id(&self, project_id: &ProjectId, sort: TrialSort) -> Result<Vec<Trial>, RepositoryError>`
  - `save(&self, trial: &Trial) -> Result<(), RepositoryError>`
  - `delete(&self, id: &TrialId) -> Result<(), RepositoryError>`
- `TrialSortColumn` は `CreatedAt`（デフォルト）と `UpdatedAt` の 2 バリアント
- `TrialSort = Sort<TrialSortColumn>` 型エイリアスを使用
- `UnitOfWork` トレイトに `type TrialRepo: TrialRepository` と `fn trial_repository(&mut self) -> Self::TrialRepo` を追加済み
- `PgUnitOfWork` には `PgTrialRepositoryStub`（todo! マクロによる仮実装）を設定済み → task 05 で `PgTrialRepository` に差し替えること
- `MockUnitOfWork` にも `MockTrialRepository`（todo! マクロによる stub）を設定済み → task 06.x でより本格的なモックに差し替えること
- `crate::ports::TrialRepository`、`crate::ports::TrialSort`、`crate::ports::TrialSortColumn` としてアクセス可能

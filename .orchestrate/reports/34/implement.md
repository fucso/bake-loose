# Sub-Issue #34 実装レポート

## 実装概要

`PgTrialRepository`（`find_by_id` / `find_all_by_project` / `save`）を実装し、Trial aggregate（Trial + Step + Parameter）を PostgreSQL に永続化・復元できるようにした。`save` は集約全体を UPSERT し、aggregate から取り除かれた Step・Parameter を削除する。あわせて `UnitOfWork` トレイト・`PgUnitOfWork`・`MockUnitOfWork` に `trial_repository()` を配線した。

なお worktree セットアップ直後、依存 Issue #32（`ports: TrialRepository トレイト定義`）がオーケストレーターのローカル `iddue/21` ブランチには完了済みだったが、`worktree-setup.sh` が `origin/iddue/21`（未 push で古い状態）を fetch したため worker worktree に反映されていない既知の不具合（`git -C <worktree> reset --hard iddue/21` で解消）を踏んだ。実装内容には影響なし。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/Cargo.toml` | sqlx に `json` feature を追加（`parameters.content` の JSONB マッピング用） |
| `backend/src/repository/models/trial_row.rs` | 新規: `TrialRow`（DBモデル）。`into_domain(steps)` で Trial へ変換、`status_column()` で TrialStatus → DBカラム値へ変換 |
| `backend/src/repository/models/step_row.rs` | 新規: `StepRow`（DBモデル）。`into_domain(parameters)` で Step へ変換（`position` は SMALLINT/i16 ⇔ i32 変換） |
| `backend/src/repository/models/parameter_row.rs` | 新規: `ParameterRow`（DBモデル）。`content` は `sqlx::types::Json<ParameterContent>` で JSONB マッピング、`From<ParameterRow> for Parameter` |
| `backend/src/repository/models.rs` | 上記3モデルの mod 登録・re-export を追加 |
| `backend/src/repository/trial_repo.rs` | 新規: `PgTrialRepository` 実装（find_by_id/find_all_by_project/save）+ 単体テスト6件 |
| `backend/src/repository.rs` | `pub mod trial_repo;` を追加 |
| `backend/src/ports/unit_of_work.rs` | `UnitOfWork` トレイトに `type TrialRepo: TrialRepository` と `fn trial_repository(&mut self) -> Self::TrialRepo` を追加 |
| `backend/src/repository/pg_unit_of_work.rs` | `PgUnitOfWork` に `TrialRepo = PgTrialRepository` と `trial_repository()` を実装 |
| `backend/src/use_case/test/mock_unit_of_work.rs` | `MockTrialRepository` を追加し `MockUnitOfWork` に配線（テスト用共通モック） |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/34
パス: .worktree/iddue/34

変更ファイル: backend/Cargo.toml
backend/src/ports/unit_of_work.rs
backend/src/repository.rs
backend/src/repository/models.rs
backend/src/repository/pg_unit_of_work.rs
backend/src/use_case/test/mock_unit_of_work.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/34/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.30s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/34/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.57s
     Running unittests src/lib.rs

running 107 tests
(...全107件 ok、省略。trial_repo::tests の6件含む全て ok...)
test repository::trial_repo::tests::test_find_by_id_returns_none_when_not_exists ... ok
test repository::trial_repo::tests::test_save_and_find_by_id_roundtrip ... ok
test repository::trial_repo::tests::test_save_updates_existing_trial ... ok
test repository::trial_repo::tests::test_find_all_by_project_returns_all_trials_for_project ... ok
test repository::trial_repo::tests::test_save_removes_parameters_not_in_aggregate ... ok
test repository::trial_repo::tests::test_save_removes_steps_not_in_aggregate ... ok

test result: ok. 107 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running unittests src/main.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/graphql.rs
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc-tests bake_loose
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

（`iddue:code-review` の Skill 呼び出しは fork 実行時に args が子エージェントへ渡らない既知の不具合があるため、SKILL.md の手順に従い手動でレビューを実施。requirements / design / code-quality の3観点とも CRITICAL・WARNING なし。code-quality で INFO 1件: `step.position()`（i32）を steps.position（SMALLINT/i16）へ `as i16` でキャストしている点について、Step数が i16::MAX を超えることは現実的に想定できないため許容と判断）

## コミット情報

- Branch: iddue/34
- Commit: 364da26254ebe35095a1f6d00e002bad20877c33
- Message: [Issue#34] repository: PgTrialRepository実装とUnitOfWork拡張

## 引き継ぎ事項

- `UnitOfWork` トレイトのシグネチャが変更された（`TrialRepo` associated type・`trial_repository()` メソッド追加）ため、`UnitOfWork` を実装する型を新たに追加する場合はこの2つの実装が必須になる。
- `PgTrialRepository::save()` は Trial aggregate 全体を UPSERT し、aggregate に存在しない Step/Parameter を削除する設計。呼び出し側（今後実装される use_case 層）は Trial 集約を丸ごと組み立ててから `save()` を呼ぶ想定（部分更新用の differential API は用意していない）。
- `parameters.content` の JSONB マッピングのため `backend/Cargo.toml` の sqlx に `json` feature を追加した。今後 JSONB 列を扱う実装でも同様に `sqlx::types::Json<T>` を利用できる。
- worker worktree セットアップ時、依存タスク完了直後は `origin/iddue/{parent}` が未 push で古い場合がある（本 Issue でも発生）。後続ワーカーは `git -C <worktree> rev-parse iddue/{parent}` と `origin/iddue/{parent}` を比較し、ずれていれば `reset --hard iddue/{parent}`（ローカル参照）で最新化してから実装を始めること。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 23:06:10 JST
- **終了:** 2026-07-30 23:16:03 JST
- **実行時間:** 9分53秒
- **消費トークン:** output 169395 / cache_read 19130312 / cache_write 490844
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/d41eb55f-b4ec-4fba-b7b8-1a1ac5786995.jsonl

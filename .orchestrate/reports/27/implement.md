# Sub-Issue #27 実装レポート

## 実装概要

Trial の name/memo を更新する `update_trial` ドメインアクションを追加した。`trial_status_validator::require_in_progress` を再利用し、Completed な Trial への更新を拒否する。Command は `name`/`memo` ともに `Option<Option<String>>` の二重 Option とし、外側 `None` で未指定（変更なし）、`Some(None)` で値のクリア、`Some(Some(v))` で値の設定を表現する部分更新パターンを採用した。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/domain/models/trial.rs` | `set_name`/`set_memo` ミューテーションメソッドを追加（テスト含む） |
| `backend/src/domain/actions/trial.rs` | 新規作成。`update_trial` モジュールを公開 |
| `backend/src/domain/actions/trial/update_trial.rs` | 新規作成。`update_trial` アクション（Command/validate/execute/run）とテスト |
| `backend/src/domain/actions.rs` | `pub mod trial;` を追加 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/27
パス: .worktree/iddue/27

変更ファイル: backend/src/domain/actions.rs
backend/src/domain/models/trial.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/27/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.50s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/27/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.60s
     Running unittests src/lib.rs (/app/target/debug/deps/bake_loose-3c55537dbfac9141)

running 44 tests
test domain::actions::project::create_project::tests::test_execute_generates_unique_id ... ok
test domain::actions::trial::update_trial::tests::test_run_can_clear_name_and_memo_to_none ... ok
test domain::actions::trial::update_trial::tests::test_run_updates_name_and_memo_when_in_progress ... ok
test domain::actions::trial::update_trial::tests::test_run_rejects_update_when_trial_completed ... ok
test domain::actions::project::create_project::tests::test_run_creates_project_with_valid_name ... ok
test domain::actions::project::create_project::tests::test_name_validation ... ok
test domain::models::project::tests::test_project_new_creates_with_auto_id ... ok
test domain::models::step::tests::test_step_id_new_generates_unique_ids ... ok
test domain::models::step::tests::test_step_new_defaults_started_at_to_now_when_unspecified ... ok
test domain::models::step::tests::test_is_completed_reflects_completed_at ... ok
test domain::models::step::tests::test_step_new_uses_specified_started_at ... ok
test domain::models::project::tests::test_project_id_new_generates_unique_ids ... ok
test domain::models::trial::tests::test_set_memo_can_set_and_clear ... ok
test domain::models::trial::tests::test_steps_mut_allows_mutating_step_by_id ... ok
test domain::models::step::tests::test_set_started_at_can_set_and_clear ... ok
test domain::models::trial::tests::test_set_name_can_set_and_clear ... ok
test domain::models::trial::tests::test_add_step_appends_to_steps ... ok
test domain::actions::trial::update_trial::tests::test_run_partially_updates_only_specified_fields ... ok
test domain::models::trial::tests::test_trial_id_new_generates_unique_ids ... ok
test domain::validators::trial::step_existence_validator::tests::test_require_exists_ok_when_step_exists ... ok
test domain::models::trial::tests::test_complete_transitions_status_to_completed ... ok
test domain::validators::trial::step_name_validator::tests::test_validate_name ... ok
test domain::validators::trial::step_status_validator::tests::test_require_in_progress_err_when_completed ... ok
test domain::models::trial::tests::test_trial_new_creates_in_progress_with_no_steps ... ok
test domain::validators::trial::step_existence_validator::tests::test_require_exists_err_when_step_not_found ... ok
test domain::validators::trial::step_status_validator::tests::test_require_in_progress_ok_when_not_completed ... ok
test domain::validators::trial::trial_status_validator::tests::test_require_in_progress_err_when_completed ... ok
test domain::validators::trial::trial_status_validator::tests::test_require_in_progress_ok_when_in_progress ... ok
test repository::project_repo::tests::test_exists_by_name_returns_false_when_not_exists ... ok
test use_case::project::create_project::tests::test_execute_creates_project_successfully ... ok
test use_case::project::create_project::tests::test_execute_returns_domain_error_for_empty_name ... ok
test use_case::project::create_project::tests::test_execute_returns_domain_error_for_too_long_name ... ok
test use_case::project::create_project::tests::test_execute_returns_duplicate_error_when_name_exists ... ok
test use_case::project::get_project::tests::test_get_project_not_found ... ok
test use_case::project::get_project::tests::test_get_project_returns_specified_project_from_multiple ... ok
test repository::project_repo::tests::test_exists_by_name_returns_true_when_exists ... ok
test use_case::project::list_projects::tests::test_list_projects_empty ... ok
test use_case::project::list_projects::tests::test_list_projects_returns_sorted_by_name_asc ... ok
test repository::project_repo::tests::test_find_all_with_name_asc ... ok
test repository::project_repo::tests::test_find_by_id_returns_project_when_exists ... ok
test repository::project_repo::tests::test_find_by_id_returns_none_when_not_exists ... ok
test repository::project_repo::tests::test_save_updates_existing_project ... ok
test repository::project_repo::tests::test_save_inserts_new_project ... ok
test repository::project_repo::tests::test_find_all_with_created_at_desc ... ok

test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s

     Running unittests src/main.rs (/app/target/debug/deps/bake_loose-82ecda5e0dcaf25f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs (/app/target/debug/deps/graphql-ce0b81d6bfc999d6)

running 8 tests
test graphql::projects::create::test_returns_error_for_empty_name ... ok
test graphql::projects::create::test_returns_error_for_duplicate_name ... ok
test graphql::projects::create::test_returns_error_for_too_long_name ... ok
test graphql::projects::get::test_returns_project ... ok
test graphql::projects::list::test_returns_empty_list ... ok
test graphql::projects::get::test_returns_null_when_not_found ... ok
test graphql::projects::create::test_creates_project_successfully ... ok
test graphql::projects::list::test_returns_projects_from_fixture ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

   Doc-tests bake_loose

running 1 test
test src/ports/unit_of_work.rs - ports::unit_of_work::UnitOfWork (line 17) ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし、WARNING なし、INFO 1件: Command の二重Optionパターンのドキュメント補足提案・任意）

## コミット情報

- Branch: iddue/27
- Commit: 704c896d3d43b7cc35e9aff2f5aab845486c9765
- Message: `[Issue#27] action: update_trial`

## 引き継ぎ事項

- 後続の use_case: update_trial（#36）は本アクションの `update_trial::Command`（`name`/`memo` は `Option<Option<String>>` の二重 Option で部分更新を表現）と `run(state, command) -> Result<Trial, update_trial::Error>` をそのまま利用できる。
- `update_trial::Error` は `trial_status_validator::Error`（`TrialAlreadyCompleted` のみ）を `pub use` で再エクスポートしている。use_case 層・presentation 層（#41）でのエラーメッセージ設計時に参照すること。
- 本アクションでは name の文字数上限バリデーションは追加していない（Issue #27 の完了条件に含まれないため）。DB カラムは `VARCHAR(100)` のため、必要であれば use_case 層または別 Issue で長さバリデーションの要否を検討すること。
- worktree セットアップ時、`worktree-setup.sh` は `origin/iddue/21` を fetch するため、オーケストレーターのローカル `iddue/21`（未 push の完了サブIssueマージを含む）より古い場合がある。本タスクでは `git reset --hard iddue/21`（ローカルブランチ参照）で最新化してから実装した。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:20:16 JST
- **終了:** 2026-07-30 22:27:03 JST
- **実行時間:** 6分47秒
- **消費トークン:** output 51566 / cache_read 7922167 / cache_write 180764
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/42723c87-12a1-4b6a-ab51-f13eebcd82f9.jsonl

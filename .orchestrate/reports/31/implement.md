# Sub-Issue #31 実装レポート

## 実装概要

指定 Step を完了状態にする `complete_step` ドメインアクションを追加した。`Step` モデルに `complete()` ミューテーションメソッドを追加し、`completed_at` が未指定の場合は `Utc::now()` を採用する（`Step::new()` の `started_at` デフォルト挙動と一貫させた）。アクションのバリデーションは既存の `trial_status_validator` / `step_existence_validator` / `step_status_validator` を再利用し、Trial 完了済み・Step 未存在・Step 完了済みの3つのエラーケースを検出する。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/domain/models/step.rs` | `Step::complete(completed_at: Option<DateTime<Utc>>)` ミューテーションメソッドを追加。未指定時は `Utc::now()` を採用。正常系2パターンのテストを追加 |
| `backend/src/domain/actions.rs` | `pub mod trial;` を追加 |
| `backend/src/domain/actions/trial.rs` | 新規。`pub mod complete_step;` |
| `backend/src/domain/actions/trial/complete_step.rs` | 新規。`complete_step` アクション（`Command` / `Error` / `validate` / `execute` / `run`）と正常系2件・異常系3件のテスト |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/31
パス: .worktree/iddue/31

変更ファイル: backend/src/domain/actions.rs
backend/src/domain/models/step.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/31/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.45s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/31/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.49s
     Running unittests src/lib.rs (/app/target/debug/deps/bake_loose-3c55537dbfac9141)

running 45 tests
(...backend既存テストを含め全て ok...)
test domain::actions::trial::complete_step::tests::test_run_err_when_step_not_found ... ok
test domain::actions::trial::complete_step::tests::test_run_err_when_trial_already_completed ... ok
test domain::actions::trial::complete_step::tests::test_run_defaults_completed_at_to_now_when_unspecified ... ok
test domain::actions::trial::complete_step::tests::test_run_completes_step_with_specified_completed_at ... ok
test domain::actions::trial::complete_step::tests::test_run_err_when_step_already_completed ... ok
test domain::models::step::tests::test_complete_defaults_completed_at_to_now_when_unspecified ... ok
test domain::models::step::tests::test_complete_uses_specified_completed_at ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s

     Running tests/graphql.rs ...
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

   Doc-tests bake_loose
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

（完全な出力は `.worktree/iddue/31/tmp/quality-check.log` 参照。cleanup 時に削除される一時ファイル）

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

- requirements: OK（INFO: `Step::complete()` 単体では「既に完了済みへの再呼び出し」を直接テストしていないが、アクション層の `step_status_validator` とそのテストでカバー済みのため要件上の欠落ではない）
- design: OK（`validate`/`execute`/`run` 分離、複数バリデーターの `Error` enum への集約、`steps_mut()` 経由のミューテーション等、`domain.md` の規約に準拠）
- code-quality: OK（重複実装・バグ・デッドコードなし）

## コミット情報

- Branch: iddue/31
- Commit: fc58f398092f1ff431d572bbd25cdfad865a4231
- Message: `[Issue#31] action: complete_step`

## 引き継ぎ事項

- 依存する use case 側（Issue #40 `use_case: complete_step`）は本アクションの `complete_step::Command { step_id, completed_at }` と `Error` enum（`TrialAlreadyCompleted` / `StepNotFound` / `StepAlreadyCompleted`）をそのまま利用できる。
- `backend/src/domain/actions/trial.rs` は本 Issue で新規作成したファイルのため、並行実装中の #26 (create_trial) / #27 (update_trial) / #28 (complete_trial) が同じファイルに `pub mod` を追加する場合はマージ時にコンフリクトが発生する可能性がある。マージ順序に応じて `pub mod` 行の統合が必要。
- **worktree セットアップ時の既知不具合**: `worktree-setup.sh` は `git fetch` してリモートの `iddue/21` を fetch した上でベースにするため、オーケストレーター側でローカルのみマージ済み（未push）の先行サブ Issue（本件では #23, #24）が worktree に反映されない場合がある。本ワーカーでは `git -C .worktree/iddue/31 reset --hard <orchestrator local iddue/21 HEAD>` で明示的に修正して対応した。次回オーケストレーション実行前に `worktree-setup.sh` の fetch 元をリモートではなくローカルの `iddue/{parent}` ブランチにする修正を検討すべき。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:20:59 JST
- **終了:** 2026-07-30 22:29:59 JST
- **実行時間:** 8分17秒
- **消費トークン:** output 58953 / cache_read 10087080 / cache_write 187068
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/1c45f4b1-46f0-4616-b256-00ea4af6b5f2.jsonl

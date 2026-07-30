# Sub-Issue #30 実装レポート

## 実装概要

既存 Step の `name`・`started_at`・`parameters`（追加・削除）を部分更新するドメインアクション `update_step` を実装した。

- `Command` は `name: Option<String>`（Some の場合のみ変更）、`started_at: Option<Option<DateTime<Utc>>>`（None: 変更なし / Some(None): クリア / Some(Some(t)): t に設定）、`add_parameters: Vec<ParameterInput>`、`remove_parameter_ids: Vec<ParameterId>` を持つ。
- `validate` は既存バリデーター（`trial_status_validator::require_in_progress` / `step_existence_validator::require_exists` / `step_status_validator::require_in_progress` / `step_name_validator::validate` / `parameter_validator::validate`）を組み合わせて Trial 完了済み・Step 未存在・Step 完了済み・不正な名前・不正なパラメーター・存在しない削除対象をエラーとして検出する。
- `execute` は `Trial::steps_mut()` で対象 Step を取得し、name/started_at の更新、parameters の追加（`Parameter::new` で子→孫の順に構築）・削除（`Step::remove_parameter`）を行う。

Issue 本文のサンプルコードでは `Error::EmptyStepName`（単一 variant）を想定していたが、実際に #24/#25 で実装された `step_name_validator::Error` は `EmptyName` / `NameTooLong` の2 variant を持つため、`parameter_validator` と同様の「ネストした reason を持つ variant」パターン（`.claude/rules/backend/domain.md` に明記された既存規約）に合わせて `Error::InvalidStepName(StepNameValidationError)` とした。要件・完了条件（エラーが返ること）自体は Issue 通り満たしている。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/domain/actions/trial/update_step.rs` | 新規追加。update_step アクション（Command/Error/validate/execute/run）と正常系・エラー系11テスト |
| `backend/src/domain/models/step.rs` | `set_name(&mut self, name: String)` ミューテーションメソッドを追加（テスト1件追加） |
| `backend/src/domain/actions/trial.rs` | `pub mod update_step;` を追加 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/30
パス: .worktree/iddue/30

変更ファイル: backend/src/domain/actions/trial.rs
backend/src/domain/models/step.rs
backend/src/domain/actions/trial/update_step.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/30/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.93s
--- cargo test ---
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src/lib.rs (/app/target/debug/deps/bake_loose-3c55537dbfac9141)

running 74 tests
（中略・全件 ok）

test result: ok. 74 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s

     Running unittests src/main.rs (/app/target/debug/deps/bake_loose-82ecda5e0dcaf25f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs (/app/target/debug/deps/graphql-ce0b81d6bfc999d6)

running 8 tests
（中略・全件 ok）

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s

   Doc-tests bake_loose

running 1 test
test src/ports/unit_of_work.rs - ports::unit_of_work::UnitOfWork (line 17) ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

（コミット後の完全なログは `tmp/quality-check.log` に保存済み。commit時点でテスト修正後に再実行した結果 11/11 update_step テストも別途確認済み）

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

`iddue:code-review` の fork 実行が引数を正しく受け取れなかったため（別途 [[feedback_worker_bash_sandbox]] 系の環境制約とは別の問題）、SKILL.md の手順（要件充足性・設計整合性・コード品質の3観点）に従い直接レビューを実施した。

| 観点 | 結果 | 備考 |
|------|------|------|
| requirements | ok | 完了条件のテストケース10件すべて実装・パス |
| design | ok | Error型のvariant構成をIssue草案から実バリデーターAPIに合わせて調整（INFO、詳細は実装概要参照） |
| code-quality | ok | clippy `-D warnings` 通過、`.expect("validated to exist")` は validate 済み前提の既存パターンに準拠 |

レビュー結果詳細: `tmp/review-result-20260730-225322.json`

## コミット情報

- Branch: iddue/30
- Commit: c3e587489261236aae87ef1fca789ab5df41b87d
- Message: [Issue#30] action: update_step

## 引き継ぎ事項

- `backend/src/domain/actions/trial.rs` に `pub mod update_step;` を追加している。並行して進行中の #29（`action: add_step`）も同じファイルに `pub mod add_step;` を追加する可能性が高く、iddue/21 へのマージ時に単純な追記行同士のコンフリクトが起きうる（内容的には両方の mod 宣言を残せばよいだけで解決は容易）。
- 後続 #39（`use_case: update_step`）はこの `update_step::run` をそのまま呼び出せる想定。`Command.started_at` が `Option<Option<DateTime<Utc>>>` である点（クリアと未指定の区別）をユースケース・GraphQL入力型の設計に反映する必要がある。
- `Error::InvalidStepName(StepNameValidationError)` および `Error::InvalidParameter { reason: ParameterValidationError, .. }` はネストしたバリデーターエラーを持つため、#39 や #41（GraphQL）でこれらをクライアント向けエラーにマッピングする際は internal な variant（EmptyName/NameTooLong, NegativeDurationValue/EmptyQuantityUnit）まで分解する実装が必要。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:45:21 JST
- **終了:** 2026-07-30 22:54:14 JST
- **実行時間:** 8分53秒
- **消費トークン:** output 102665 / cache_read 14975575 / cache_write 434780
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/984fbdc2-7710-416b-b538-0d5c03afe1a1.jsonl

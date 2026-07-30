# Sub-Issue #36 実装レポート

## 実装概要

`use_case: update_trial` を実装した。trial_id で Trial を取得し、`update_trial` ドメインアクション（#27 で実装済み）を適用して `TrialRepository`（#34 で実装済み）経由で保存する use case を追加した。`.claude/rules/backend/use-case.md` のパターン（`begin()` → DB取得 → ドメインアクション → `save()` → `commit()`、エラー時 `rollback()`）に従っている。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/use_case.rs` | `pub mod trial;` を追加 |
| `backend/src/use_case/trial.rs` | 新規追加。`pub mod update_trial;` |
| `backend/src/use_case/trial/update_trial.rs` | 新規追加。`Input`（trial_id + `update_trial::Command`）、`Error`（NotFound / Domain / Infrastructure）、`execute()` を実装。正常更新・NotFound・ドメインエラー伝播（Trial完了済み）の3テストを追加 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/36
パス: .worktree/iddue/36

変更ファイル: backend/src/use_case.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/36/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.04s
--- cargo test ---
    Compiling bake-loose v0.1.0 (/worktrees/iddue/36/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.50s
     Running unittests src/lib.rs

running 110 tests
test result: ok. 110 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s

     Running unittests src/main.rs

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs

running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s

   Doc-tests bake_loose

running 1 test
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

新規追加テスト（use_case::trial::update_trial）:
test use_case::trial::update_trial::tests::test_execute_returns_domain_error_when_trial_completed ... ok
test use_case::trial::update_trial::tests::test_execute_returns_not_found_when_trial_does_not_exist ... ok
test use_case::trial::update_trial::tests::test_execute_updates_name_and_memo_successfully ... ok

結果: fmt/clippy/test すべて成功
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

- 要件充足性: 完了条件（正常更新・NotFound・ドメインエラー伝播のテスト）を全て満たしパス済み
- 設計整合性: use-case.md のパターンに厳密準拠、依存 Issue #27/#34 の型定義と整合
- コード品質: domain.md/ports.md/testing.md の規約に適合、fmt/clippy/test すべて成功

## コミット情報

- Branch: iddue/36
- Commit: 9821967
- Message: [Issue#36] use_case: update_trial

## 引き継ぎ事項

- GraphQL 層（presentation）からの呼び出しは未実装。後続でミューテーションリゾルバーを追加する Issue が `use_case::trial::update_trial::execute` を呼び出す想定。
- `Input.command` は `update_trial::Command`（`name: Option<Option<String>>`, `memo: Option<Option<String>>`）をそのまま利用しており、GraphQL 側の Input 型からのマッピングが必要になる。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 23:18:13 JST
- **終了:** 2026-07-30 23:26:18 JST
- **実行時間:** 8分5秒
- **消費トークン:** output 46424 / cache_read 10219460 / cache_write 271512
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/1fb79d1f-b346-4ba0-ad94-26b93cf406ea.jsonl

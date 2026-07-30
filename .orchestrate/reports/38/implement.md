# Sub-Issue #38 実装レポート

## 実装概要

`trial_id` で Trial を取得し、`add_step` ドメインアクション（依存Issue #29、既にマージ済み）を適用・保存する use_case `use_case::trial::add_step` を追加した。トランザクション境界（begin/commit/rollback）を UnitOfWork 経由で管理し、Trial が見つからない場合は `Error::NotFound`、ドメインアクションのバリデーション失敗（Trial完了済み・不正なStep名・不正なParameter）は `Error::Domain` として伝播する。use-case.md のパターンに従い、ドメインアクションの `add_step::Command` 型を Input にそのまま再利用した。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/use_case.rs` | `pub mod trial;` を追加 |
| `backend/src/use_case/trial.rs` | Trial ユースケースの集約モジュールを新規作成（`pub mod add_step;`） |
| `backend/src/use_case/trial/add_step.rs` | add_step use_case本体（Input/Error/execute）とテスト4件を新規作成 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/38
パス: .worktree/iddue/38

変更ファイル: backend/src/use_case.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/38/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.84s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/38/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.42s
     Running unittests src/lib.rs (/app/target/debug/deps/bake_loose-3c55537dbfac9141)

running 111 tests
test result: ok. 111 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.40s

     Running unittests src/main.rs (/app/target/debug/deps/bake_loose-82ecda5e0dcaf25f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs (/app/target/debug/deps/graphql-ce0b81d6bfc999d6)

running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s

   Doc-tests bake_loose

running 1 test
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

--- 結果まとめ ---
- cargo fmt --check: OK
- cargo clippy --all-targets -- -D warnings: OK（警告なし）
- cargo test: OK（111 + 0 + 8 unit/integration tests, 1 doc-test ignored）
- use_case::trial::add_step の新規テスト4件（正常系・NotFound・TrialAlreadyCompleted・InvalidStepName）すべて成功
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

（`iddue:code-review` の Skill fork 実行で args が渡らない既知の不具合があるため、SKILL.md の手順に従い手動でレビューを実施。requirements / design / code-quality の3観点すべて `ok`。詳細は `tmp/review-result-20260730-232242.json` 相当の内容。）

## コミット情報

- Branch: iddue/38
- Commit: 2e98b4c950337726f4ccc42ebf8cd6bb3a104cda
- Message: [Issue#38] use_case: add_step

## 引き継ぎ事項

- worktree-setup.sh はブランチを `origin/iddue/21`（リモートの stale な状態）からセットアップしてしまい、ローカル worktree `.worktree/iddue/21` の最新コミット（#34 PgTrialRepository 等を含む）が反映されていなかった。今回は `git reset --hard iddue/21`（ローカルブランチ参照）で修正してから実装した。後続のサブ Issue ワーカーも同様の問題に遭遇する可能性があるため、オーケストレーター側でこの点に注意すること。
- このブランチ系譜には `.gitignore` の `/tmp/*` エントリが反映される前の分岐点から派生しているため、`.worktree/iddue/38/tmp/` 配下が gitignore されない。既に `tmp/review-result-*.json` が複数、祖先コミットに誤って commit 済みだった（今回のコミットには含めていない）。親Issue #21 マージ時に一括で `tmp/` 配下の誤コミットファイルを削除するクリーンアップを検討してほしい。
- 本 use_case はまだ GraphQL 層（presentation）や PgUnitOfWork への配線が行われていない。他のサブ Issue で mutation resolver 実装時にこの `use_case::trial::add_step::execute` を呼び出す想定。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 23:18:17 JST
- **終了:** 2026-07-30 23:24:34 JST
- **実行時間:** 6分17秒
- **消費トークン:** output 74836 / cache_read 13540647 / cache_write 299814
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/bedc3c59-5650-4da5-b8f3-23db11e505a8.jsonl

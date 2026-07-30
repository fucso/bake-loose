# Sub-Issue #39 実装レポート

## 実装概要

trial_id で Trial を取得し、`domain::actions::trial::update_step` ドメインアクションを適用して永続化する use_case（`use_case::trial::update_step`）を追加した。UnitOfWork 経由でトランザクションを開始し、Trial 未存在時は `Error::NotFound`、ドメインアクションのバリデーション失敗時は `Error::Domain` としてエラーを伝播する。永続化・インフラ層のエラーは `Error::Infrastructure` にラップする。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/use_case.rs` | `pub mod trial;` を追加 |
| `backend/src/use_case/trial.rs` | 新規作成。`pub mod update_step;` |
| `backend/src/use_case/trial/update_step.rs` | 新規作成。update_step use_case本体と正常系・NotFound・ドメインエラー伝播のテスト（4件）を実装 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/39
パス: .worktree/iddue/39

変更ファイル: backend/src/use_case.rs
--- cargo fmt --check ---
（差分なし）
--- cargo clippy --all-targets -- -D warnings ---
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.10s
--- cargo test ---
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.07s
     Running unittests src/lib.rs

running 111 tests
test use_case::trial::update_step::tests::test_propagates_domain_error_when_step_not_found ... ok
test use_case::trial::update_step::tests::test_propagates_domain_error_when_trial_completed ... ok
test use_case::trial::update_step::tests::test_update_step_name_success ... ok
test use_case::trial::update_step::tests::test_returns_not_found_when_trial_does_not_exist ... ok
... (他107件省略、全て ok)

test result: ok. 111 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s

     Running unittests src/main.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

   Doc-tests bake_loose
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

`iddue:code-review` の Skill 呼び出しが args 未到達で機能しなかったため（既知の不具合）、SKILL.md の手順（requirements / design / code-quality の各観点ファイル）に従い手動でレビューを実施した。要件（正常系・NotFound・ドメインエラー伝播のテスト）を充足し、既存の `use_case/project/*.rs` の設計パターン（validate/execute分離のドメインアクション呼び出し、UnitOfWork経由のトランザクション管理）と整合していることを確認済み。

## コミット情報

- Branch: iddue/39
- Commit: 8432e5a
- Message: [Issue#39] use_case: update_step

## 引き継ぎ事項

- 本 use_case はまだ GraphQL 層（presentation）から呼び出されていない。Step の name/started_at/parameters を更新する mutation resolver の実装は別サブ Issue の対象。
- 依存Issue #30（action: update_step）・#34（PgTrialRepository実装）はいずれも `iddue/21` にマージ済みで、本実装はそのコードをそのまま利用している。
- worker worktree セットアップ時、`worktree-setup.sh` が `origin/iddue/21` を fetch base にするため、ローカル `iddue/21`（オーケストレーター worktree）に積まれた未push の依存コミット（#30, #34のマージ）が worker worktree に反映されない既知の問題があった。`git -C .worktree/iddue/39 reset --hard iddue/21` で最新化してから実装を開始した。後続ワーカーも同様の確認が必要な可能性がある。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 23:18:07 JST
- **終了:** 2026-07-30 23:24:52 JST
- **実行時間:** 6分45秒
- **消費トークン:** output 40970 / cache_read 13597574 / cache_write 231928
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/89df5623-269f-4f85-924e-faddf389f72b.jsonl

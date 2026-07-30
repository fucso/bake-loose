# Sub-Issue #23 実装レポート

## 実装概要

trials / steps / parameters の3テーブルを作成するマイグレーションファイルを `backend/migrations/` 配下に3ファイル追加した。

- `trials`: `project_id` FK（`projects` 参照）、`name`/`memo` は nullable、`status` は `VARCHAR(20) NOT NULL DEFAULT 'in_progress'`、`idx_trials_project_id` インデックス
- `steps`: `trial_id` FK（`ON DELETE CASCADE`）、`name` NOT NULL、`position` は `SMALLINT`（PR #18 レビューで `i32` は過剰と指摘されたため）、`UNIQUE(trial_id, position)`、`started_at`/`completed_at` は nullable
- `parameters`: `step_id` FK（`ON DELETE CASCADE`）、`content` は `JSONB NOT NULL`、`idx_parameters_step_id` インデックス（コードレビュー指摘を受けて追加）

スキーマは PR #18（`feature/20260228-trial_model` ブランチ、ドメイン層実装）の `Trial`/`Step`/`Parameter` モデル定義（`i16` position, `Option<String>` name/memo 等）と整合させた。マイグレーションファイルは AGENTS.md の命名規則（1操作1対象）に従い3ファイルに分割した。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/migrations/20260730000000_create_trials.sql` | `trials` テーブル新規作成 |
| `backend/migrations/20260730000001_create_steps.sql` | `steps` テーブル新規作成 |
| `backend/migrations/20260730000002_create_parameters.sql` | `parameters` テーブル新規作成 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/23
パス: .worktree/iddue/23

変更ファイル: backend/migrations/20260730000000_create_trials.sql, backend/migrations/20260730000001_create_steps.sql, backend/migrations/20260730000002_create_parameters.sql
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/23/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.28s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/23/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.24s
     Running unittests src/lib.rs
running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

     Running unittests src/main.rs
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

   Doc-tests bake_loose
running 1 test
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

補足: バックエンドのテストは `#[sqlx::test]` マクロを使用しており、テスト実行時に `backend/migrations/` 配下の全マイグレーションを新規テスト用DBに適用してからテストを実行する。全テストが成功したことで、新規追加した3マイグレーションファイルが構文エラーなく適用可能であることを確認済み。

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

1回目のレビューで `code-quality` 観点から WARNING（`parameters.step_id` に対応するインデックスがなく `trials.project_id`/`steps.trial_id` との一貫性に欠ける）が1件あったため、`idx_parameters_step_id` を追加し品質チェックを再実行して成功を確認した。CRITICAL 指摘は0件だったため再レビューは実施していない。

## コミット情報

- Branch: iddue/23
- Commit: 726298b
- Message: `[Issue#23] migration: trials/steps/parameters テーブル作成`

## 引き継ぎ事項

- Issue #34（`repository: PgTrialRepository実装とUnitOfWork拡張`）は本 Issue に依存する。マイグレーション適用後、`trials`/`steps`/`parameters` テーブルに対して SQLx クエリを実装できる状態になっている。
- `position` カラムは `SMALLINT`（Rust側は `i16`）である点に注意。ドメイン層・リポジトリ層の実装でも `i16` を使用すること（PR #18 のレビュー方針と統一）。
- `steps.trial_id`・`parameters.step_id` は `ON DELETE CASCADE` のため、Trial/Step 削除時に子テーブルの手動削除処理は不要。
- スキーマの参考として `feature/20260228-trial_model` ブランチ（未マージ）に類似のドメインモデル実装があるが、`position` の型（当該ブランチは `INTEGER`）が本 Issue の決定（`SMALLINT`）と異なるため、実装時は本マイグレーションの型定義を正とすること。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:03:25 JST
- **終了:** 2026-07-30 22:12:44 JST
- **実行時間:** 9分19秒
- **消費トークン:** output 48133 / cache_read 10751332 / cache_write 180481
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/5e2deb9c-4e55-4972-a36f-b230cc6783d3.jsonl

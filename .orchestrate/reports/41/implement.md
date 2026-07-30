# Sub-Issue #41 実装レポート

## 実装概要

Trial/Step/Parameter の GraphQL 型、6つの mutation リゾルバー（createTrial / updateTrial / completeTrial / addStep / updateStep / completeStep）、2つの query リゾルバー（trial / trialsByProject）を実装し、Project 実装と同じ構成（薄いリゾルバー・ラッパー型・`UserFacingError` によるエラー変換）でスキーマに統合した。

query リゾルバーが利用する `get_trial` / `list_trials_by_project` use_case は未実装だったため、既存の `get_project` / `list_projects` と同じパターンで新規追加した。

Trial の `name`/`memo`、Step の `started_at` のような「未指定=変更なし・null=クリア」を区別する更新は `async_graphql::MaybeUndefined` を用いて GraphQL 入力から `Option<Option<T>>` に変換している。`ParameterContent`（KeyValue/Duration/TimeMarker/Text の tagged union）は Issue の決定事項どおり `async_graphql::Json<ParameterContent>` でそのまま JSON スカラーとして入出力する設計とし、Union 型の分岐実装は行っていない。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/presentation/graphql/types/trial.rs` | 新規追加。`Trial`/`Step`/`Parameter` ラッパー型、`TrialStatus` Enum、`CreateTrialInput`/`UpdateTrialInput`/`AddStepInput`/`UpdateStepInput` |
| `backend/src/presentation/graphql/mutation/trial.rs` | 新規追加。`TrialMutation`（6つの mutation リゾルバー） |
| `backend/src/presentation/graphql/query/trial.rs` | 新規追加。`TrialQuery`（`trial`/`trialsByProject`） |
| `backend/src/presentation/graphql/error.rs` | Trial 関連 use_case エラー（create_trial/update_trial/complete_trial/add_step/update_step/complete_step/get_trial/list_trials_by_project）の `UserFacingError` 実装を追加 |
| `backend/src/presentation/graphql/types.rs` / `mutation.rs` / `query.rs` / `schema.rs` | `trial` サブモジュールの登録、`QueryRoot`/`MutationRoot` への `TrialQuery`/`TrialMutation` の統合 |
| `backend/src/use_case/trial/get_trial.rs` | 新規追加。IDでTrialを取得する読み取り専用 use_case |
| `backend/src/use_case/trial/list_trials_by_project.rs` | 新規追加。プロジェクトに紐づくTrial一覧を取得する読み取り専用 use_case |
| `backend/src/use_case/trial.rs` | 上記2つの use_case モジュールを登録 |
| `backend/Cargo.toml` | `async-graphql` に `chrono` feature を追加（`DateTime<Utc>` をGraphQLスカラーとして使うため） |
| `backend/tests/fixtures/trials.sql` | 新規追加。GraphQLテスト用のTrialフィクスチャ（in_progress×3、completed×1） |
| `backend/tests/graphql/trials.rs`, `trials/{create,update,complete,steps,get,list,lifecycle}.rs` | 新規追加。各 mutation/query の統合テストと、Trial作成〜Step追加〜完了までの一連のライフサイクルテスト |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/41
パス: .worktree/iddue/41

変更ファイル: backend/Cargo.lock
backend/Cargo.toml
backend/src/presentation/graphql/error.rs
backend/src/presentation/graphql/mutation.rs
backend/src/presentation/graphql/query.rs
backend/src/presentation/graphql/schema.rs
backend/src/presentation/graphql/types.rs
backend/src/use_case/trial.rs
backend/tests/graphql.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
--- cargo test ---
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src/lib.rs (/app/target/debug/deps/bake_loose-89d3c0ed6675f832)

running 132 tests
test result: ok. 132 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s

     Running unittests src/main.rs (/app/target/debug/deps/bake_loose-d0894354a891f5c0)

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs (/app/target/debug/deps/graphql-a7cca8dc4b4b45c2)

running 31 tests
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s

   Doc-tests bake_loose

running 1 test
test src/ports/unit_of_work.rs - ports::unit_of_work::UnitOfWork (line 18) ... ignored
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` ともに警告・エラーなし。既存128件 + 新規4件（get_trial×2, list_trials_by_project×2）のユニットテストと、新規31件（Project 8件 + Trial 23件）のGraphQL統合テストがすべて成功。

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

- requirements: OK — Issue の完了条件（Trial作成〜Step追加〜完了までの一連のGraphQL操作、cargo test全体成功）を `trials/lifecycle.rs` の e2e テストと `cargo test` 全体成功で充足
- design: OK — Project実装と同一パターン（薄いリゾルバー・ラッパー型・UserFacingErrorによるエラー変換）に準拠、スコープ外の変更なし。参考情報として、`trials/list.rs` の複数件テストはDB行順序が保証されないため名前ソートで比較する設計とした点を INFO で記録（単一件のケースは完全なJSON一致で検証）
- code-quality: OK — 重大な問題なし（review-insights は取得不可のため参照なしで判定）

## コミット情報

- Branch: iddue/41
- Commit: d08abca95ddbf800432a25d7d3a41f3d41ef8f86（実装コミット）、続けて d83aa77（tmp/ 誤コミットを除去する chore コミット）
- Message: `[Issue#41] presentation: Trial GraphQL スキーマ・リゾルバー`

## 引き継ぎ事項

- 依存 Issue #35〜#40（use_case: create_trial/update_trial/complete_trial/add_step/update_step/complete_step）はすべて base ブランチ（iddue/21）上に実装済みだったが、着手時点で **#37（complete_trial）のみ orchestrator の状態ファイル上は `completed` 扱いながら、iddue/21 ブランチにはまだマージされていなかった**（`orchestration: complete sub-issues #36 #39 #40, reset crashed #37 to pending` の後、#37 の再実行が完了した直後というタイミング）。ローカルに残っていた完了済みの `iddue/37` ブランチ（`report: sub-issue #37 implementation complete` 済み）を本ブランチへ fast-forward merge して依存関係を満たした。オーケストレーターが後続で `iddue/37` を `iddue/21` にマージする際、本 Issue のマージ順序次第では重複マージや衝突が起きないか確認すること（fast-forward マージのため差分自体は同一のはずだが、念のため）。
- `ParameterContent` は Union 型ではなく `async_graphql::Json` によるJSONスカラーとして公開している（Issue の決定事項どおり）。フロントエンド側は `type` フィールドで分岐する必要がある。
- `trialsByProject` はソート順を持たない（`TrialRepository::find_all_by_project` 自体に ORDER BY がないため）。表示順が必要な場合はフロントエンド側でのソート、または別 Issue でのソート機能追加が必要。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 23:51:19 JST
- **終了:** 2026-07-31 00:09:38 JST
- **実行時間:** 18分19秒
- **消費トークン:** output 179779 / cache_read 37515867 / cache_write 438359
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/fb130259-453d-46da-b1ba-33a5b2c4b6fb.jsonl

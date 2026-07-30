# Sub-Issue #47 実装レポート

## 実装概要

frontend にルーティング・共通レイアウト構造を導入した。react-router-dom の `createBrowserRouter` /
`RouterProvider` を `main.tsx` に組み込み、`/` と `/projects` のプレースホルダーページを共通レイアウト
（ヘッダー・ナビゲーション）配下に表示できるようにした。既存の `App.tsx`（バックエンドヘルスチェック表示）
は廃止し、その内容は `HomePage`（`/`）に移設した。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `frontend/src/main.tsx` | `<App />` の直接レンダリングを `<RouterProvider router={router} />` に変更 |
| `frontend/src/routes/router.tsx` | `createBrowserRouter` によるルート定義（`/` → `RootLayout` + `index: HomePage`、`/projects` → `ProjectsPage`）を新規追加 |
| `frontend/src/layouts/RootLayout.tsx` | 共通レイアウト（ヘッダー・ナビゲーション + `Outlet`）を新規追加 |
| `frontend/src/pages/HomePage.tsx` | `App.tsx` をリネームし、旧トップレベル見出しをレイアウト側に譲る形で調整（バックエンドヘルスチェック表示を維持） |
| `frontend/src/pages/ProjectsPage.tsx` | `/projects` のプレースホルダーページを新規追加 |
| `frontend/src/App.tsx` | 削除（`HomePage.tsx` へ移設） |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/47
パス: .worktree/iddue/47

変更ファイル: frontend/src/App.tsx
frontend/src/main.tsx
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/47/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.96s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/47/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.39s
     Running unittests src/lib.rs (/app/target/debug/deps/bake_loose-3c55537dbfac9141)

running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running unittests src/main.rs (/app/target/debug/deps/bake_loose-82ecda5e0dcaf25f)

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs (/app/target/debug/deps/graphql-ce0b81d6bfc999d6)

running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

   Doc-tests bake_loose

running 1 test
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**補足:** このリポジトリの `.claude/worktree-hooks/quality-check.sh` は backend（Rust）のみを対象としており、
frontend の型チェック・ビルドは自動化されていない。本 Issue はフロントエンドのみの変更のため、
worktree の frontend ディレクトリに対して以下を手動実行し確認した（`docker compose exec` は frontend
コンテナのマウント設定（`.agents/worktrees` を参照しており `.worktree/` を参照していない）の都合で使えないため、
起動済み frontend コンテナの `node_modules` 匿名ボリュームを再利用した `docker run` で検証）。

```
$ pnpm exec tsc -b --noEmit
TSC_OK

$ pnpm build
> tsc -b && vite build
vite v6.4.3 building for production...
✓ 43 modules transformed.
dist/index.html                  0.32 kB │ gzip:  0.24 kB
dist/assets/index-BBdq0JUv.js  287.97 kB │ gzip: 92.46 kB
✓ built in 1.64s
BUILD_OK

$ vite preview で / と /projects への到達を確認
- GET /         → 200
- GET /projects → 200（SPA フォールバックで index.html を返却、react-router がクライアント側でマッチ）
- <title>bake-loose</title> を確認
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

INFO レベルの改善提案が1件あったが、Issue の完了条件では要求されていないため対応不要と判断した：
- `frontend/src/routes/router.tsx`: `createBrowserRouter` にルート単位の `errorElement` / 404 ハンドリングが
  未設定。未定義パスへのアクセス時は react-router のデフォルトエラー UI が表示され、共通レイアウトが適用されない。

## コミット情報

- Branch: iddue/47
- Commit: 0cc1ebb4eefa76be20b801f76657168ef4b54d22
- Message: [Issue#47] frontend: ルーティング・レイアウト構造の導入

## 引き継ぎ事項

- 以降の画面実装（Project一覧・Trial記録フォーム等）は `src/routes/router.tsx` にルートを追加し、
  `src/pages/` にページコンポーネントを追加する形で本構造に組み込む。
- `/projects` は現時点ではプレースホルダー表示のみ。実データ表示は別 Issue で対応する想定。
- 404/未定義パスのハンドリング（`errorElement`）は本 Issue の完了条件外のため未実装。必要になった時点で
  `router.tsx` に追加すること。
- このリポジトリの frontend 用 Docker Compose 設定は `compose.yaml` の `frontend` サービスが
  `./.agents/worktrees:/worktrees` をマウントしており、backend サービスの `./.worktree:/worktrees` と
  食い違っている。そのため `.claude/worktree-hooks/exec.sh`（backend コンテナ経由専用）では frontend の
  worktree コードを検証できない。今後 frontend 向けの quality-check フックを整備する際はこのマウント不整合を
  先に修正するか、host 側から `docker run` で直接コンテナを起動する方式を検討する必要がある。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:56:30 JST
- **終了:** 2026-07-30 23:07:34 JST
- **実行時間:** 11分4秒
- **消費トークン:** output 71234 / cache_read 10657115 / cache_write 187784
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/e8346135-3da9-4592-9b85-8141736a87b2.jsonl

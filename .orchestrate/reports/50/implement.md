# Sub-Issue #50 実装レポート

## 実装概要

frontend に `vite-plugin-pwa`（Workbox ベース）を導入し、PWA としてインストール可能な状態にした。
`vite.config.ts` に `VitePWA` プラグインを設定し、アプリ名・アイコン・テーマカラーを含むマニフェストと、
SPA 向けの `navigateFallback` を含む基本的な Workbox キャッシュ戦略（`generateSW` モード、ビルド出力の
プリキャッシュ）を構成した。マニフェストで参照するアイコン（192x192 / 512x512 / maskable 512x512 /
apple-touch-icon）は `frontend/public/` に追加し、`index.html` に `theme-color` メタタグと
`apple-touch-icon` の `link` タグを追加した。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `frontend/package.json` | `vite-plugin-pwa` を devDependencies に追加 |
| `frontend/pnpm-lock.yaml` | `pnpm install` 実行により新規生成（新規追加） |
| `frontend/vite.config.ts` | `VitePWA` プラグインを追加し、manifest・workbox（`navigateFallback`/`globPatterns`）・`devOptions.enabled` を設定 |
| `frontend/index.html` | `theme-color` メタタグ、`apple-touch-icon` の `link` タグを追加 |
| `frontend/public/pwa-192x192.png` | マニフェスト用アイコン（192x192、新規追加） |
| `frontend/public/pwa-512x512.png` | マニフェスト用アイコン（512x512、新規追加） |
| `frontend/public/maskable-icon-512x512.png` | マニフェスト用 maskable アイコン（新規追加） |
| `frontend/public/apple-touch-icon.png` | iOS ホーム画面用アイコン（180x180、新規追加） |
| `.gitignore` | `frontend/dist/` `frontend/dev-dist/` `frontend/node_modules/` `frontend/.pnpm-store/` `frontend/tsconfig.tsbuildinfo` を追加（ビルド確認時に生成される成果物の誤コミット防止） |

## 品質チェック結果

`worktree-quality-check.sh`（backend 向け: `cargo fmt --check` → `cargo clippy` → `cargo test`）は本 Issue の
変更範囲（frontend）に影響しないため全項目 OK。加えて、backend コンテナに Node.js が無く既存の
`quality-check.sh` フックが frontend を検証対象にしていないため、`docker compose run --rm -v
<worktree>/frontend:/app frontend sh -c "pnpm install && pnpm run build"` を個別に実行し
`tsc -b && vite build` が型エラーなく成功すること、`dist/manifest.webmanifest` と `dist/sw.js`
（Service Worker）が生成されることを確認済み。

```
=== 品質チェック ===
環境名: iddue/50
パス: .worktree/iddue/50

変更ファイル: .gitignore
frontend/index.html
frontend/package.json
frontend/vite.config.ts
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.52s
--- cargo test ---
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
running 21 tests ... test result: ok. 21 passed; 0 failed
running 0 tests (main.rs) ... test result: ok. 0 passed; 0 failed
running 8 tests (graphql.rs) ... test result: ok. 8 passed; 0 failed
Doc-tests bake_loose ... test result: ok. 0 passed; 0 failed; 1 ignored

--- frontend（追加検証、backend 用 quality-check.sh の対象外のため個別実施）---
docker compose run --rm -e CI=true -v <worktree>/frontend:/app frontend sh -c "pnpm install && pnpm run build"
> tsc -b && vite build
✓ 28 modules transformed.
PWA v1.3.0 mode generateSW / precache 13 entries (198.50 KiB)
files generated: dist/sw.js, dist/workbox-9c191d2f.js, dist/manifest.webmanifest, dist/registerSW.js
→ 型エラーなくビルド成功、manifest.webmanifest / Service Worker の生成を確認済み。

注: frontend/eslint.config.js がリポジトリに存在しないため `pnpm run lint` は本 Issue 着手前から
失敗する既存の環境ギャップであり、本 Issue の変更によるものではないため対応対象外とした。
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

`iddue:code-review` を Skill ツール経由で fork 実行すると引数が子エージェントに渡らない既知の不具合
（過去のワーカー実行で確認済み）があるため、観点ファイル（`requirements.md` / `design.md` /
`code-quality.md`）の手順に従い手動でレビューを実施した。要件充足性・設計整合性はいずれも OK。
コード品質観点で 1 件 INFO（`frontend/vite.config.ts`: バックエンド API のランタイムキャッシュは
GraphQL クライアント導入（#48）後にエンドポイント仕様が固まってから追加するのが妥当、との補足）を
記録したが CRITICAL/WARNING はなし。結果は
`.worktree/iddue/50/tmp/review-result-20260730-230700.json` を参照。

## コミット情報

- Branch: iddue/50
- Commit: 2bf308fbc965fb8b734d2514a12dc6846419838c
- Message: [Issue#50] frontend: PWA(Workbox)対応

## 引き継ぎ事項

- frontend の依存関係管理は `pnpm`（Dockerfile が `corepack prepare pnpm@10` を使用）だが、これまで
  `pnpm-lock.yaml` が存在しなかった。本 Issue の実装時に初めて生成してコミットしたため、以後の
  frontend 変更（Issue #47/#48/#49 等）はこのロックファイルを更新しつつ進めること。
- `compose.yaml` の `frontend` サービスは `./frontend:/app` と `./.agents/worktrees:/worktrees` を
  マウントしており、`.worktree/` 配下のワーカー worktree を直接対象にできない（`backend` サービス向け
  `.claude/worktree-hooks/exec.sh` も frontend の Node コマンド実行には対応していない）。本 Issue では
  `docker compose run --rm -v <worktree>/frontend:/app frontend ...` で一時的にボリュームを上書きして
  ビルド検証したが、恒久的には frontend 用の worktree-hooks 対応（またはマウント構成の見直し）が
  望ましい。
- `VitePWA` の `workbox` 設定は `generateSW` モードのデフォルト（ビルド出力のプリキャッシュ + SPA 用
  `navigateFallback`）のみで、バックエンド API に対するランタイムキャッシュ戦略（NetworkFirst 等）は
  未設定。GraphQL クライアント（Issue #48）導入後、実際のエンドポイント仕様に合わせて追加を検討する。
- `devOptions.enabled: true` により `vite dev` 実行時にも Service Worker が有効化される
  （`frontend/dev-dist/` が生成されるが `.gitignore` 済み）。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:56:36 JST
- **終了:** 2026-07-30 23:07:59 JST
- **実行時間:** 11分23秒
- **消費トークン:** output 70000 / cache_read 15039720 / cache_write 195278
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/7b6a1492-0f3f-43b3-933f-561a258b0390.jsonl

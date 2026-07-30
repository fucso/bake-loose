# Sub-Issue #49 実装レポート

## 実装概要

frontend（Vite + React）に Tailwind CSS と shadcn/ui を導入した。

- Tailwind CSS v4（`@tailwindcss/vite` プラグイン方式）を導入し、`vite.config.ts` に `tailwindcss()` プラグインを追加
- shadcn/ui CLI（`shadcn@latest init`）を非対話モードで実行し、`components.json` / `src/lib/utils.ts` / テーマ用 CSS 変数を含む `src/index.css` を生成
- `@/*` パスエイリアスを `tsconfig.json`（`baseUrl` / `paths`）と `vite.config.ts`（`resolve.alias`）に追加（shadcn/ui のコンポーネント import に必要）
- 動作確認用に shadcn/ui の `Button` コンポーネントを追加し、`App.tsx` の「再確認」ボタンとして実際に使用（クリックでヘルスチェックを再実行）
- `frontend/node_modules` / `dist` / `.pnpm-store` / `*.tsbuildinfo` が未整備だったため `.gitignore` に追記（このリポジトリで frontend の依存関係インストールを行ったのは本 Issue が初めてで、追記しないと大量の生成物が誤ってコミットされる状態だった）

**設計からの意図的な逸脱（要確認）:**
Issue 本文の実装内容には `tailwind.config` / `postcss.config` を用いた Tailwind CSS v3 系のセットアップ手順が記載されていたが、実際には shadcn/ui の現行公式 Vite 導入手順に合わせて **Tailwind CSS v4 + `@tailwindcss/vite`** 方式（`tailwind.config.js` / `postcss.config.js` 不要、`index.css` に `@import "tailwindcss";` のみ）で実装した。完了条件（Tailwind ユーティリティクラスの適用・shadcn Button のレンダリング・ビルド成功）はすべて満たしている。

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `.gitignore` | `node_modules/` `dist/` `.pnpm-store/` `*.tsbuildinfo` を追加 |
| `frontend/vite.config.ts` | `@tailwindcss/vite` プラグイン追加、`@/*` エイリアス設定 |
| `frontend/tsconfig.json` | `baseUrl` / `paths`（`@/*`）を追加 |
| `frontend/src/main.tsx` | `./index.css` の import 追加 |
| `frontend/src/App.tsx` | Tailwind ユーティリティクラスへ置き換え、shadcn/ui `Button` を導入して動作確認用「再確認」ボタンを追加 |
| `frontend/package.json` | `tailwindcss` `@tailwindcss/vite` `shadcn` `class-variance-authority` `clsx` `tailwind-merge` `lucide-react` `@base-ui/react` `@fontsource-variable/geist` `tw-animate-css` を追加（shadcn CLI が自動追加） |
| `frontend/components.json`（新規） | shadcn/ui CLI 設定ファイル |
| `frontend/pnpm-lock.yaml`（新規） | 依存関係ロックファイル |
| `frontend/src/index.css`（新規） | Tailwind エントリポイント + shadcn/ui テーマ CSS 変数 |
| `frontend/src/lib/utils.ts`（新規） | shadcn/ui の `cn()` ヘルパー |
| `frontend/src/components/ui/button.tsx`（新規） | shadcn/ui `Button` コンポーネント |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/49
パス: .worktree/iddue/49

変更ファイル: .gitignore
frontend/package.json
frontend/src/App.tsx
frontend/src/main.tsx
frontend/tsconfig.json
frontend/vite.config.ts
frontend/components.json (新規)
frontend/pnpm-lock.yaml (新規)
frontend/src/index.css (新規)
frontend/src/components/ui/button.tsx (新規)
frontend/src/lib/utils.ts (新規)

--- cargo fmt --check (backend, 変更なし・回帰確認用) ---
(差分なし)

--- cargo clippy --all-targets -- -D warnings (backend) ---
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s

--- cargo test (backend) ---
test result: ok. 21 passed; 0 failed; 0 ignored (unit tests)
test result: ok. 8 passed; 0 failed; 0 ignored (graphql integration tests)
test result: ok. 0 passed; 0 failed; 1 ignored (doc-tests)

--- pnpm run build (frontend, tsc -b && vite build) ---
> bake-loose-frontend@0.0.1 build /app
> tsc -b && vite build

vite v6.4.3 building for production...
transforming...
✓ 58 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                                             0.40 kB │ gzip:  0.27 kB
dist/assets/geist-cyrillic-ext-wght-normal-DjL33-gN.woff2    7.42 kB
dist/assets/geist-vietnamese-wght-normal-6IgcOCM7.woff2      8.00 kB
dist/assets/geist-cyrillic-wght-normal-BEAKL7Jp.woff2       15.08 kB
dist/assets/geist-latin-ext-wght-normal-DC-KSUi6.woff2      16.51 kB
dist/assets/geist-latin-wght-normal-BgDaEnEv.woff2          29.40 kB
dist/assets/index-DnVlDpkk.css                              22.55 kB │ gzip:  4.85 kB
dist/assets/index-D9OsO8-g.js                               234.40 kB │ gzip: 73.95 kB
✓ built in 5.80s

型エラーなし・ビルド成功（Tailwind CSS のユーティリティクラスが index-*.css として出力されていることを確認済み）。

--- pnpm run lint (frontend, eslint) ---
既知の問題（本 Issue のスコープ外・修正なし）:
ESLint couldn't find an eslint.config.(js|mjs|cjs) file.
→ このリポジトリの frontend には元々 eslint.config.js が存在せず、ESLint v9 のフラットコンフィグ移行が
   未実施のため lint コマンド自体が実行できない状態だった（本 Issue の変更前から存在する既存の欠落）。
   Tailwind CSS / shadcn/ui 導入とは無関係のため本 Issue では対応せず、引き継ぎ事項に記録する。
```

補足: `.claude/worktree-hooks/quality-check.sh` は backend（cargo）専用実装であり frontend（pnpm）には未対応のため、frontend のビルド検証は `docker compose run --rm --no-deps` で `frontend` サービスをワークツリーにバインドマウントして手動実行した。

## レビュー結果

✅ レビュー OK（全観点 CRITICAL なし）

- `requirements`: OK — 完了条件（Tailwind 適用・Button レンダリング・ビルド成功）をすべて充足
- `design`: OK（WARNING 1件）— 上記「設計からの意図的な逸脱」を参照。Tailwind v4 方式への変更は意図的かつ完了条件を満たすため WARNING に留める
- `code-quality`: OK（INFO 1件）— shadcn CLI が `shadcn` パッケージを `dependencies` に追加する既定動作について、本来は `devDependencies` が望ましいが CLI のデフォルト挙動であり実害なし

## コミット情報

- Branch: iddue/49
- Commit: 0bfb945976863253f3560317c98c9b5f7c92e10b
- Message: [Issue#49] frontend: Tailwind CSS / shadcn/ui 導入

## 引き継ぎ事項

- **frontend の ESLint 設定が未整備**: `eslint.config.js` が存在せず `pnpm run lint` が実行できない状態（本 Issue 着手前からの既存ギャップ）。以降のフロントエンド作業で lint を機能させるには ESLint v9 フラットコンフィグの追加が別途必要。
- **shadcn/ui のスタイルは `base-nova`**（CLI のデフォルト選択、`-d` オプションによる非対話初期化のため）。デザインの方向性が既に決まっている場合は `components.json` の `style` / `baseColor` を見直すこと。
- **`.claude/worktree-hooks/exec.sh` / `quality-check.sh` は backend（cargo）専用**で frontend の pnpm コマンド実行に対応していない。また `compose.yaml` の `frontend` サービスは `./frontend:/app` を固定マウントしており worktree 単位のバインドができず、かつ `/app/node_modules` の匿名ボリュームが node_modules への実際のバインドマウントを隠蔽してしまう（`docker compose run` 時に `-v <worktree>/frontend/node_modules:/app/node_modules` で明示的に上書きしないと `pnpm install` の内容が永続化されない）。以降のフロントエンド系サブ Issue でも同じ問題に当たるため、worktree-hooks 側を frontend 対応させる、または `compose.yaml` の frontend サービス定義を worktree 対応にする恒久対応が望ましい。
- 依存関係バージョン: `tailwindcss@4.3.3`, `@tailwindcss/vite@4.3.3`, `shadcn@4.16.0`。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:56:46 JST
- **終了:** 2026-07-30 23:10:28 JST
- **実行時間:** 13分42秒
- **消費トークン:** output 97506 / cache_read 18030457 / cache_write 270392
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/77f588c9-b0ca-4810-a542-4b89d17ecc32.jsonl

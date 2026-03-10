# Scripts Reference

このスキルが提供するすべてのスクリプトの完全なリファレンス。

## Table of Contents

- [Worktree Management](#worktree-management)
- [Docker Integration](#docker-integration)
- [Git Operations](#git-operations)
- [Library Functions](#library-functions)

---

## Worktree Management

### setup-worktree.sh

新しい worktree とブランチを作成します。

**シグネチャ:**
```bash
setup-worktree.sh <branch-name> [base-branch]
```

**パラメータ:**
- `branch-name` (必須): 作成するブランチ名
- `base-branch` (オプション): ベースとなるブランチ（省略時は現在のブランチ）

**出力:**
- 作成された worktree のパス

**例:**
```bash
# 現在のブランチから作成
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/new-feature)

# main ブランチから作成
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/new-feature main)

# develop ブランチから作成
WORKTREE_PATH=$(scripts/setup-worktree.sh fix/bug-123 develop)
```

**エラー:**
- 引数不足: `Usage: setup-worktree.sh <branch-name> [base-branch]`
- ブランチ名が既に存在: git のエラーメッセージを表示

**内部動作:**
1. リポジトリルートを取得
2. ブランチ名から worktree ディレクトリ名を生成（スラッシュ → アンダースコア）
3. `.worktrees/<worktree-dir>` に worktree を作成
4. パスを標準出力に出力

---

### close-worktree.sh

Worktree を削除し、オプションでブランチも削除します。

**シグネチャ:**
```bash
close-worktree.sh <worktree-path> [--delete-branch]
```

**パラメータ:**
- `worktree-path` (必須): 削除する worktree のパス
- `--delete-branch` (オプション): ブランチも削除する

**例:**
```bash
# Worktree のみ削除
scripts/close-worktree.sh /path/to/.worktrees/feature_new-feature

# Worktree とブランチを削除
scripts/close-worktree.sh /path/to/.worktrees/feature_new-feature --delete-branch
```

**エラー:**
- 引数不足: `Usage: close-worktree.sh <worktree-path> [--delete-branch]`
- worktree が存在しない: git のエラーメッセージを表示
- ブランチがマージされていない: `git branch -d` のエラーメッセージを表示

**内部動作:**
1. worktree のブランチ名を取得
2. worktree を削除（`git worktree remove --force`）
3. `--delete-branch` が指定されている場合、ブランチを削除（`git branch -d`）

**注意:**
- このスクリプトはマージを行いません
- マージされていないブランチを削除しようとするとエラーになります
- 強制削除したい場合は手動で `git branch -D` を実行してください

---

## Docker Integration

### docker-exec.sh

Docker Compose exec のラッパー。worktree 内で任意のコマンドを実行します。

**シグネチャ:**
```bash
docker-exec.sh <service> <command> [subdir]
```

**パラメータ:**
- `service` (必須): Docker サービス名（backend, frontend など）
- `command` (必須): 実行するコマンド
- `subdir` (オプション): worktree 内のサブディレクトリ

**環境変数:**
- `MAIN_REPO_PATH` (必須): メインリポジトリのパス
- `DOCKER_WORKTREE_PATH` (必須): Docker コンテナ内の worktree パス

**例:**
```bash
export MAIN_REPO_PATH="/path/to/project"
export DOCKER_WORKTREE_PATH="/worktrees/feature_new-feature"

# ビルド
docker-exec.sh backend "cargo build"

# backend サブディレクトリでテスト
docker-exec.sh backend "cargo test" backend

# シェル起動
docker-exec.sh backend "bash"
```

**エラー:**
- 引数不足: `Usage: docker-exec.sh <service> <command> [subdir]`
- 環境変数未設定: `Error: MAIN_REPO_PATH must be set` または `Error: DOCKER_WORKTREE_PATH must be set`
- サービスが存在しない: Docker Compose のエラーメッセージを表示

**内部動作:**
1. 環境変数をチェック
2. 作業ディレクトリを決定（`DOCKER_WORKTREE_PATH/subdir`）
3. `docker compose -f ${MAIN_REPO_PATH}/compose.yaml exec <service> bash -c "cd <work_dir> && <command>"` を実行

---

### build.sh

バックエンドのビルドを実行します。

**シグネチャ:**
```bash
build.sh
```

**環境変数:**
- `MAIN_REPO_PATH` (必須)
- `DOCKER_WORKTREE_PATH` (必須)

**例:**
```bash
export MAIN_REPO_PATH="/path/to/project"
export DOCKER_WORKTREE_PATH="/worktrees/feature_new-feature"

scripts/build.sh
```

**内部実装:**
```bash
docker-exec.sh backend "cargo build"
```

---

### test.sh

バックエンドのテストを実行します。

**シグネチャ:**
```bash
test.sh
```

**環境変数:**
- `MAIN_REPO_PATH` (必須)
- `DOCKER_WORKTREE_PATH` (必須)

**例:**
```bash
export MAIN_REPO_PATH="/path/to/project"
export DOCKER_WORKTREE_PATH="/worktrees/feature_new-feature"

scripts/test.sh
```

**内部実装:**
```bash
docker-exec.sh backend "cargo test"
```

---

### lint.sh

バックエンドの Lint を実行します。

**シグネチャ:**
```bash
lint.sh
```

**環境変数:**
- `MAIN_REPO_PATH` (必須)
- `DOCKER_WORKTREE_PATH` (必須)

**例:**
```bash
export MAIN_REPO_PATH="/path/to/project"
export DOCKER_WORKTREE_PATH="/worktrees/feature_new-feature"

scripts/lint.sh
```

**内部実装:**
```bash
docker-exec.sh backend "cargo clippy -- -D warnings"
```

---

## Git Operations

### commit.sh

Worktree 内でコミットを実行します。

**シグネチャ:**
```bash
commit.sh <type> <message> <files...>
```

**パラメータ:**
- `type` (必須): コミットタイプ（feat, fix, refactor, test, docs, report）
- `message` (必須): コミットメッセージ
- `files...` (必須): コミットするファイル（複数可）

**環境変数:**
- `WORKTREE_PATH` (必須): worktree のパス
- `FEATURE_ID` (必須): フィーチャー識別子（ブランチ名など）

**例:**
```bash
export WORKTREE_PATH="/path/to/.worktrees/feature_new-feature"
export FEATURE_ID="feature/new-feature"

# 新機能のコミット
scripts/commit.sh feat "Add user authentication" src/auth/

# バグ修正のコミット
scripts/commit.sh fix "Fix login validation" src/auth/login.rs

# 複数ファイルのコミット
scripts/commit.sh feat "Add authentication system" src/auth/ src/middleware/
```

**エラー:**
- 引数不足: `Usage: commit.sh <type> <message> <files...>`
- 環境変数未設定: `Error: WORKTREE_PATH must be set` または `Error: FEATURE_ID must be set`

**コミットメッセージ形式:**
```
<type>(<feature-id>): <message>
```

**内部動作:**
1. 環境変数をチェック
2. ファイルを git add（`git -C ${WORKTREE_PATH} add <files>`）
3. コミット実行（`git -C ${WORKTREE_PATH} commit -m "<type>(<feature-id>): <message>"`）

---

## スクリプトのパス解決

スクリプトを呼び出す際は、以下のいずれかの方法でパスを解決します:

### 1. 相対パス（推奨）

メインリポジトリから呼び出す場合:

```bash
.claude/skills/background-developing-with-worktree/scripts/setup-worktree.sh
```

### 2. 絶対パス

環境変数を使用:

```bash
export SKILL_DIR="${MAIN_REPO_PATH}/.claude/skills/background-developing-with-worktree"
${SKILL_DIR}/scripts/build.sh
```

### 3. スクリプトディレクトリに移動

```bash
cd .claude/skills/background-developing-with-worktree
./scripts/setup-worktree.sh feature/new-feature
```

---

## エラーハンドリング

すべてのスクリプトは `set -euo pipefail` を使用しており:

- エラー発生時に即座に終了
- 未定義変数の使用を禁止
- パイプライン内のエラーを検出

エラーメッセージは標準エラー出力（stderr）に出力されます。

---

## デバッグ

スクリプトのデバッグには以下の方法を使用できます:

### bash -x でトレース

```bash
bash -x scripts/setup-worktree.sh feature/test
```

### set -x を追加

スクリプトに一時的に `set -x` を追加:

```bash
#!/bin/bash
set -euo pipefail
set -x  # デバッグモード

# ... スクリプトの内容 ...
```

### 環境変数の確認

```bash
echo "WORKTREE_PATH: ${WORKTREE_PATH}"
echo "MAIN_REPO_PATH: ${MAIN_REPO_PATH}"
echo "DOCKER_WORKTREE_PATH: ${DOCKER_WORKTREE_PATH}"
```

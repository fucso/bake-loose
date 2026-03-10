# Workflow Guide

Git worktree を使用した開発ワークフローの詳細ガイド。

## Table of Contents

- [Quick Start](#quick-start)
- [Workflow Decision Tree](#workflow-decision-tree)
- [Worktree Management](#worktree-management)
- [Commit Workflow](#commit-workflow)

---

## Quick Start

### 1. Worktree のセットアップ

```bash
# 新しいブランチと worktree を作成（.worktree.env が自動生成される）
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/new-feature)

# ベースブランチを指定して作成
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/new-feature main)
```

### 2. 作業

```bash
cd ${WORKTREE_PATH}

# 実装
# ...

# ビルド・テスト（.worktree.env から設定を自動読み込み）
scripts/build.sh
scripts/test.sh
```

### 3. クローズ

```bash
# Worktree とブランチを削除
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

---

## Workflow Decision Tree

```
新しい開発タスクがある
    ↓
メインリポジトリに影響を与えたくない？
    ├─ Yes → Worktree をセットアップ
    └─ No  → メインリポジトリで直接作業

Worktree で作業
    ↓
Docker でビルド・テストが必要？
    ├─ Yes → docker-integration.md を参照
    └─ No  → 通常通りコミット

作業完了
    ↓
変更をマージする必要がある？
    ├─ Yes → メインリポジトリでマージ後、Worktree をクローズ
    └─ No  → Worktree をクローズ（ブランチは保持可能）
```

---

## Worktree Management

### セットアップ

`scripts/setup-worktree.sh` を使用して新しい worktree を作成します。

**基本形式:**
```bash
WORKTREE_PATH=$(scripts/setup-worktree.sh <branch-name> [base-branch])
```

**出力:**
- 作成された worktree のパス（`.worktrees/<branch-name>` 形式）

**例:**
```bash
# 現在のブランチから新しいブランチを作成
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/user-auth)

# main ブランチから新しいブランチを作成
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/user-auth main)

# バグ修正用のブランチを作成
WORKTREE_PATH=$(scripts/setup-worktree.sh fix/login-bug develop)
```

### クローズ

`scripts/close-worktree.sh` を使用して worktree を削除します。

**基本形式:**
```bash
scripts/close-worktree.sh <worktree-path> [--delete-branch]
```

**オプション:**
- なし: worktree のみ削除（ブランチは保持）
- `--delete-branch`: worktree とブランチを両方削除

**例:**
```bash
# Worktree のみ削除（後でマージしたい場合）
scripts/close-worktree.sh ${WORKTREE_PATH}

# Worktree とブランチを削除（作業を破棄する場合）
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

**重要:** このスクリプトはマージを行いません。マージは別途実行してください。

---

## Commit Workflow

Worktree 内でコミットを行うには、`scripts/commit.sh` を使用します。

**基本形式:**
```bash
scripts/commit.sh <type> <message> <files...>
```

**type の種類:**
- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `test`: テスト追加
- `docs`: ドキュメント

**例:**
```bash
# .worktree.env から設定を自動読み込み

# 実装のコミット
scripts/commit.sh feat "Add user authentication" src/auth/

# バグ修正のコミット
scripts/commit.sh fix "Fix login validation" src/auth/login.rs
```

---

## Complete Examples

実際の開発シーンでの完全なワークフロー例は [examples.md](examples.md) を参照してください:

- 新機能開発
- 並列開発
- 実験的な変更
- バグ修正
- Docker を使った開発
- 複数コミットのワークフロー
- トラブル発生時の対応

---

## Important Notes

### Worktree の配置

- Worktree は `.worktrees/` ディレクトリに作成されます
- `.gitignore` に `.worktrees/` を追加することを推奨します

### 設定ファイル（.worktree.env）

各 worktree には `.worktree.env` ファイルが自動生成されます:

```bash
# .worktree.env の例
WORKTREE_PATH="/path/to/.worktrees/feature_new-feature"
MAIN_REPO_PATH="/path/to/main/repo"
DOCKER_WORKTREE_PATH="/worktrees/feature_new-feature"
BRANCH_NAME="feature/new-feature"
FEATURE_ID="feature/new-feature"
```

**利点:**
- 環境変数の手動設定が不要
- スクリプトが自動的に設定を読み込む
- worktree ごとに独立した設定

**注意:**
- `.worktree.env` は worktree 内に作成されるため、`.gitignore` への追加は不要
- スクリプトは worktree ディレクトリから実行する必要があります

### ブランチの管理

- Worktree を削除してもブランチは残ります（`--delete-branch` を使用しない限り）
- マージは手動で行う必要があります
- ブランチ名のスラッシュは worktree ディレクトリ名でアンダースコアに変換されます

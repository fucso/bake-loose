---
name: background-developing-with-worktree
description: |
  Git worktree を使用したバックグラウンド開発環境の管理。メインリポジトリに影響を与えずに、独立した作業ディレクトリで機能開発やバグ修正を行うためのワークフロー。
  使用タイミング: (1) 新機能や実験的な変更を分離して開発したい時、(2) 並列で複数のタスクに取り組みたい時、(3) Docker Compose 環境で worktree 内のコードをビルド・テストしたい時、(4) 一時的な作業環境が必要な時
---

# Background Developing with Worktree

Git worktree を使用して、メインリポジトリから分離された独立した開発環境を構築・管理するスキル。

## Overview

このスキルは、git worktree を使用した分離開発環境の構築から、Docker Compose でのビルド・テスト、クリーンアップまでの一連のワークフローを提供します。

**提供機能:**
- Worktree の作成と削除
- Docker Compose でのビルド・テスト実行
- Worktree 内での Git 操作（コミット）

## Quick Start

```bash
# 1. Worktree セットアップ（.worktree.env が自動生成される）
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/new-feature main)

# 2. 作業ディレクトリに移動
cd ${WORKTREE_PATH}
# ... 実装 ...

# 3. ビルド・テスト（.worktree.env から設定を自動読み込み）
scripts/build.sh
scripts/test.sh

# 4. コミット（.worktree.env から設定を自動読み込み）
scripts/commit.sh feat "Add new feature" src/

# 5. クローズ
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

詳細なワークフローは [references/workflow-guide.md](references/workflow-guide.md) を参照してください。

## Resources

### scripts/

| カテゴリ | スクリプト | 説明 |
|---------|-----------|------|
| **Worktree 管理** | `setup-worktree.sh` | worktree とブランチの作成 |
| | `close-worktree.sh` | worktree の削除 |
| **Docker 統合** | `docker-exec.sh` | Docker Compose exec のラッパー |
| | `build.sh`, `test.sh`, `lint.sh` | ビルド・テスト・Lint 実行 |
| **Git 操作** | `commit.sh` | worktree 内でのコミット |

詳細は [references/scripts-reference.md](references/scripts-reference.md) を参照してください。

### references/

| ドキュメント | 内容 |
|------------|------|
| [workflow-guide.md](references/workflow-guide.md) | ワークフロー詳細、Decision Tree |
| [examples.md](references/examples.md) | 完全なワークフロー例（新機能開発、並列開発、バグ修正など） |
| [docker-integration.md](references/docker-integration.md) | Docker Compose 統合の詳細、トラブルシューティング |
| [scripts-reference.md](references/scripts-reference.md) | スクリプトの完全なリファレンス |

## Important Notes

- **配置**: Worktree は `.worktrees/` ディレクトリに作成されます
- **設定ファイル**: `.worktree.env` が各 worktree に自動生成され、パスなどの設定を保持します
- **Docker 設定**: `compose.yaml` に `.worktrees:/worktrees` のマウント設定が必要

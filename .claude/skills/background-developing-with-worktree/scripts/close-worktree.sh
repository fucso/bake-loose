#!/bin/bash
# worktree をクローズする（worktree削除 + オプションでブランチ削除）
# マージは行わない（別途 git merge を実行すること）
# 引数: worktree-path [--delete-branch]
# 使用例:
#   ./close-worktree.sh /path/to/.worktrees/feature_new-feature
#   ./close-worktree.sh /path/to/.worktrees/feature_new-feature --delete-branch
#
# 注意: このスクリプトはメインリポジトリから実行してください。
#       worktree 内から実行した場合、自動的にメインリポジトリに移動します。

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <worktree-path> [--delete-branch]" >&2
  exit 1
fi

WORKTREE_PATH="$1"
DELETE_BRANCH=false

if [[ $# -ge 2 ]] && [[ "$2" == "--delete-branch" ]]; then
  DELETE_BRANCH=true
fi

# worktreeのブランチ名を取得
BRANCH_NAME=$(git -C "${WORKTREE_PATH}" branch --show-current)

# メインリポジトリのパスを取得
# worktree 内にいる場合は .worktree.env から、そうでなければ git rev-parse から取得
if [[ -f "${WORKTREE_PATH}/.worktree.env" ]]; then
  # shellcheck disable=SC1091
  source "${WORKTREE_PATH}/.worktree.env"
  MAIN_REPO="${MAIN_REPO_PATH}"
else
  MAIN_REPO=$(git rev-parse --show-toplevel 2>/dev/null || echo "")
fi

# カレントディレクトリがメインリポジトリでない場合は移動
CURRENT_DIR=$(pwd)
if [[ "${CURRENT_DIR}" == "${WORKTREE_PATH}"* ]]; then
  echo "Warning: Currently in worktree directory. Moving to main repository..." >&2
  if [[ -n "${MAIN_REPO}" ]]; then
    cd "${MAIN_REPO}"
  else
    echo "Error: Could not determine main repository path" >&2
    exit 1
  fi
fi

# worktree削除
git worktree remove "${WORKTREE_PATH}" --force

# ブランチ削除（オプション）
if [[ "${DELETE_BRANCH}" == "true" ]]; then
  git branch -d "${BRANCH_NAME}"
fi

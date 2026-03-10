#!/bin/bash
# .worktree.env ファイルを読み込む共通関数
# 使用例: source "$(dirname "$0")/load-worktree-env.sh"

set -euo pipefail

# .worktree.env の場所を探す
# 1. カレントディレクトリ
# 2. スクリプト実行元のディレクトリ
# 3. git worktree のルート

find_worktree_env() {
  local search_dirs=(
    "."
    "$(pwd)"
  )

  # git worktree のルートも探す
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    local worktree_root
    worktree_root=$(git rev-parse --show-toplevel)
    search_dirs+=("${worktree_root}")
  fi

  for dir in "${search_dirs[@]}"; do
    if [[ -f "${dir}/.worktree.env" ]]; then
      echo "${dir}/.worktree.env"
      return 0
    fi
  done

  return 1
}

# .worktree.env を探して読み込む
WORKTREE_ENV_FILE=$(find_worktree_env)

if [[ -z "${WORKTREE_ENV_FILE}" ]]; then
  echo "Error: .worktree.env not found" >&2
  echo "Please run this script from a worktree directory, or ensure .worktree.env exists" >&2
  exit 1
fi

# .worktree.env を読み込む
# shellcheck disable=SC1090
source "${WORKTREE_ENV_FILE}"

# 必要な変数が設定されているか確認
if [[ -z "${WORKTREE_PATH:-}" ]]; then
  echo "Error: WORKTREE_PATH not set in .worktree.env" >&2
  exit 1
fi

if [[ -z "${MAIN_REPO_PATH:-}" ]]; then
  echo "Error: MAIN_REPO_PATH not set in .worktree.env" >&2
  exit 1
fi

if [[ -z "${DOCKER_WORKTREE_PATH:-}" ]]; then
  echo "Error: DOCKER_WORKTREE_PATH not set in .worktree.env" >&2
  exit 1
fi

if [[ -z "${BRANCH_NAME:-}" ]]; then
  echo "Error: BRANCH_NAME not set in .worktree.env" >&2
  exit 1
fi

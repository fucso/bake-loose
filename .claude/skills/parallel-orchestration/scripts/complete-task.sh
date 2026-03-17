#!/bin/bash
# complete-task.sh
#
# タスク完了後のクリーンアップ処理を一括で行うスクリプト。
# オーケストレーターがマージ完了後に使用する。
#
# 注意: マージはこのスクリプトの前にオーケストレーターが行う。
#       このスクリプトはマージ後のクリーンアップのみを担当する。
#
# 使い方:
#   bash complete-task.sh <task-id>
#
# 引数:
#   task-id  - 完了したタスクの ID（例: 01-migration）
#
# 自動取得:
#   REPO_ROOT      - common/get-repo-root.sh
#   FEATURE_ID     - common/get-active-feature-id.sh
#   WORKTREE_PATH  - tasks.js worktreePath から取得
#   BRANCH_NAME    - tasks.js branch から取得
#
# 処理:
#   1. status.yaml から worktree パスとブランチ名を取得
#   2. worktree を削除（background-developing-with-worktree skill に委譲）
#   3. タスクブランチを削除
#   4. status.yaml を更新（active → completed）
#
# 出力:
#   stdout に完了メッセージを出力
#
# エラー:
#   引数不足:        exit 1
#   情報取得失敗:    exit 2
#   worktree削除失敗: exit 3

set -euo pipefail

# --- 引数チェック ---

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <task-id>" >&2
  exit 1
fi

TASK_ID="$1"

# --- 共通情報の取得 ---

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

REPO_ROOT=$("${SCRIPT_DIR}/common/get-repo-root.sh") || {
  echo "Error: not inside a git repository" >&2
  exit 2
}

FEATURE_ID=$("${SCRIPT_DIR}/common/get-active-feature-id.sh") || {
  echo "Error: failed to get active feature id" >&2
  exit 2
}

if [ -z "$FEATURE_ID" ]; then
  echo "Error: no active feature in .agents/active.yaml" >&2
  exit 2
fi

# --- status.yaml から worktree パスとブランチ名を取得 ---

RELATIVE_WORKTREE_PATH=$(node "${SCRIPT_DIR}/tasks.js" worktreePath "${TASK_ID}") || {
  echo "Error: failed to get worktree path for task ${TASK_ID}" >&2
  exit 2
}

BRANCH_NAME=$(node "${SCRIPT_DIR}/tasks.js" branch "${TASK_ID}") || {
  echo "Error: failed to get branch name for task ${TASK_ID}" >&2
  exit 2
}

WORKTREE_PATH="${REPO_ROOT}/${RELATIVE_WORKTREE_PATH}"

# --- worktree 削除（background-developing-with-worktree skill に委譲）---

CLOSE_SCRIPT="${REPO_ROOT}/.claude/skills/background-developing-with-worktree/scripts/close-worktree.sh"

bash "${CLOSE_SCRIPT}" "${WORKTREE_PATH}" || {
  echo "Error: failed to close worktree at ${WORKTREE_PATH}" >&2
  exit 3
}

# --- タスクブランチを削除 ---

cd "${REPO_ROOT}"
git branch -d "${BRANCH_NAME}"

# --- status.yaml 更新（active → completed）---

node "${SCRIPT_DIR}/tasks.js" toCompleted "${TASK_ID}"

echo "Task ${TASK_ID} completed successfully"

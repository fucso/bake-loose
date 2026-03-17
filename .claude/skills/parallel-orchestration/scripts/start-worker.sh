#!/bin/bash
# start-worker.sh
#
# 指定タスクの worktree を作成し、ワーカープロセスを起動するスクリプト。
# オーケストレーターがディスパッチ時に使用する。
#
# worktree のセットアップは background-developing-with-worktree skill に委譲する。
# setup-worktree.sh が .worktree.env を自動生成し、パス情報を提供する。
#
# 使い方:
#   bash start-worker.sh <task-id>
#
# 引数:
#   task-id  - 起動するタスクの ID（例: 01-migration）
#
# 自動取得:
#   REPO_ROOT      - common/get-repo-root.sh
#   FEATURE_ID     - common/get-active-feature-id.sh
#   WORKTREE_PATH  - setup-worktree.sh の出力
#   DOCKER_WORKTREE_PATH - .worktree.env から読み込み
#
# 処理:
#   1. worktree をセットアップ（background-developing-with-worktree skill に委譲）
#   2. ワーカー用プロンプトを構築
#   3. claude -p をバックグラウンドで起動
#   4. status.yaml を更新（pending → active）
#
# 出力:
#   stdout にワーカーの PID を出力
#
# エラー:
#   引数不足:        exit 1
#   情報取得失敗:    exit 2
#   worktree作成失敗: exit 3
#   ワーカー起動失敗: exit 4

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

# --- worktree 作成（background-developing-with-worktree skill に委譲）---

BRANCH_NAME="task/${FEATURE_ID}_${TASK_ID}"
SETUP_SCRIPT="${REPO_ROOT}/.claude/skills/background-developing-with-worktree/scripts/setup-worktree.sh"

WORKTREE_PATH=$(bash "${SETUP_SCRIPT}" "${BRANCH_NAME}") || {
  echo "Error: failed to create worktree for branch ${BRANCH_NAME}" >&2
  exit 3
}

# .worktree.env から Docker パスを読み込み
# shellcheck disable=SC1091
source "${WORKTREE_PATH}/.worktree.env"

# --- パス構築 ---

TASK_DIR="${REPO_ROOT}/.agents/features/${FEATURE_ID}/tasks/${TASK_ID}"
LOG_PATH="${TASK_DIR}/worker_output.log"

# --- プロンプト構築 ---

PROMPT=$(cat <<EOF
以下のタスクを実行してください。

## タスク情報

- タスク仕様: .agents/features/${FEATURE_ID}/tasks/${TASK_ID}/spec.md
- Feature 仕様: .agents/features/${FEATURE_ID}/spec.md
- ワーカーガイド: .claude/skills/parallel-orchestration/references/worker-behavior.md

## 環境変数

- WORKTREE_PATH: ${WORKTREE_PATH}
- MAIN_REPO_PATH: ${REPO_ROOT}
- DOCKER_WORKTREE_PATH: ${DOCKER_WORKTREE_PATH}
- FEATURE_ID: ${FEATURE_ID}
- TASK_ID: ${TASK_ID}

## 実行手順

1. AGENTS.md を読み込む
2. ワーカーガイド（worker-behavior.md）を読み込む
3. タスク仕様（spec.md）を読み込む
4. 依存タスクの report.md があれば参照する
5. 対象レイヤーのコーディングルールを参照する（.claude/rules/ 配下）
6. 実装を行う
7. ビルド・テストを実行する
8. 実装コードをコミットする
9. report.md を作成し、別コミットとしてコミットする

## 制約

- report.md 以外の .agents/ 配下のファイルは編集しない
- タスク仕様に記載された範囲のみを実装する
- Docker Compose 環境は共有されている（他ワーカーと同時実行の可能性あり）

## 完了条件

タスクディレクトリに report.md を作成・コミットすることで完了を通知する。
EOF
)

# --- ワーカー起動 ---

(
  export WORKTREE_PATH
  export MAIN_REPO_PATH="${REPO_ROOT}"
  export DOCKER_WORKTREE_PATH
  export FEATURE_ID
  export TASK_ID

  cd "${WORKTREE_PATH}" && env -u CLAUDECODE claude -p "${PROMPT}" --dangerously-skip-permissions > "${LOG_PATH}" 2>&1
) &

WORKER_PID=$!

if ! kill -0 "$WORKER_PID" 2>/dev/null; then
  echo "Error: failed to start worker process" >&2
  exit 4
fi

# --- status.yaml 更新（pending → active）---

# worktree_path はリポジトリルートからの相対パスに変換
RELATIVE_WORKTREE_PATH="${WORKTREE_PATH#"${REPO_ROOT}/"}"

node "${SCRIPT_DIR}/tasks.js" toActive "${TASK_ID}" "${RELATIVE_WORKTREE_PATH}" "${BRANCH_NAME}" "${WORKER_PID}"

echo "$WORKER_PID"

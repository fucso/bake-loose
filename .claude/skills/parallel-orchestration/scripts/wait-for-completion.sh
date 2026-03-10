#!/bin/bash
# wait-for-completion.sh
#
# アクティブなタスクの完了を監視し、最初に完了したタスクを検知して exit するスクリプト。
# オーケストレーター（Claude）が run_in_background=true で起動し、
# 完了通知を受けて merge/dispatch 処理を行う設計。
#
# 使い方:
#   bash wait-for-completion.sh
#
# 引数: なし
#
# 自動取得:
#   REPO_PATH      - common/get-repo-root.sh
#   FEATURE_ID     - common/get-active-feature-id.sh
#   ACTIVE_TASKS   - tasks.js active
#
# ポーリング間隔:
#   初回 30秒、以降試行ごとに +15秒（最大 300秒）
#
# 出力:
#   完了検知時:    stdout に "COMPLETED:<task_id>" を出力して exit 0
#   クラッシュ検知時: stdout に "CRASHED:<task_id>" を出力して exit 1
#   エラー時:      stderr にメッセージを出力して exit 2
#
# 完了判定:
#   タスクブランチに report.md がコミットされていること
#   git show task/{feature-id}_{task-id}:.agents/features/{feature-id}/tasks/{task-id}/report.md
#
# クラッシュ判定:
#   ワーカープロセス（claude -p）が存在せず、かつ report.md も未コミット

set -euo pipefail

# --- ポーリング設定 ---

POLL_INITIAL=30   # 初回間隔（秒）
POLL_FACTOR=15    # 毎回加算する秒数
POLL_MAX=300      # 最大間隔（秒）

# --- 共通情報の取得 ---

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

REPO_PATH=$("${SCRIPT_DIR}/common/get-repo-root.sh") || {
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

# --- Active Tasks の取得 ---

mapfile -t ACTIVE_TASKS < <(node "${SCRIPT_DIR}/tasks.js" active)

if [ ${#ACTIVE_TASKS[@]} -eq 0 ]; then
  echo "Error: no active tasks in status.yaml" >&2
  exit 2
fi

# --- ヘルパー関数 ---

# タスクブランチに report.md がコミットされているか確認
check_report_committed() {
  local task_id="$1"
  local branch="task/${FEATURE_ID}_${task_id}"
  local report_path=".agents/features/${FEATURE_ID}/tasks/${task_id}/report.md"

  git -C "$REPO_PATH" show "${branch}:${report_path}" > /dev/null 2>&1
}

# タスクのワーカープロセスが生きているか確認
# worktree パスを含む claude プロセスの存在で判定
check_worker_alive() {
  local task_id="$1"
  local worktree_name="${FEATURE_ID}_${task_id}"

  pgrep -f "claude.*${worktree_name}" > /dev/null 2>&1
}

# --- メインループ ---

poll_interval=$POLL_INITIAL

while true; do
  for task_id in "${ACTIVE_TASKS[@]}"; do
    # 完了チェック: report.md がコミットされているか
    if check_report_committed "$task_id"; then
      echo "COMPLETED:${task_id}"
      exit 0
    fi

    # クラッシュチェック: プロセスが死んでいて report.md もない
    if ! check_worker_alive "$task_id"; then
      echo "CRASHED:${task_id}"
      exit 1
    fi
  done

  sleep "$poll_interval"

  # 次回の間隔を増加（上限あり）
  poll_interval=$(( poll_interval + POLL_FACTOR ))
  if [ "$poll_interval" -gt "$POLL_MAX" ]; then
    poll_interval=$POLL_MAX
  fi
done

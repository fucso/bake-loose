#!/bin/bash
# worktree 内でコミットを実行
# .worktree.env から設定を読み込む
#
# 引数:
#   $1: type (feat | fix | report | refactor | test | docs)
#   $2: message
#   $@: files (残りの引数)
#
# 使用例:
#   ./commit.sh feat "Add Trial model" src/domain/
#   ./commit.sh report "完了レポート" .agents/features/.../report.md

set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "Usage: $0 <type> <message> <files...>" >&2
  exit 1
fi

TYPE="$1"
MESSAGE="$2"
shift 2
FILES=("$@")

# .worktree.env を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/load-worktree-env.sh"

git -C "${WORKTREE_PATH}" add "${FILES[@]}"
git -C "${WORKTREE_PATH}" commit -m "${TYPE}(${FEATURE_ID}): ${MESSAGE}"

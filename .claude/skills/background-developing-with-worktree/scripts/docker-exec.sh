#!/bin/bash
# Docker Compose exec のラッパー
# .worktree.env から設定を読み込む
#
# 引数:
#   $1: service (backend | frontend)
#   $2: command
#   $3: subdir (オプション)
#
# 使用例:
#   ./docker-exec.sh backend "cargo build"
#   ./docker-exec.sh backend "cargo test" some/subdir

set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <service> <command> [subdir]" >&2
  exit 1
fi

SERVICE="$1"
COMMAND="$2"
SUBDIR="${3:-}"

# .worktree.env を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/load-worktree-env.sh"

# 作業ディレクトリの決定
if [[ -n "${SUBDIR}" ]]; then
  WORK_DIR="${DOCKER_WORKTREE_PATH}/${SUBDIR}"
else
  WORK_DIR="${DOCKER_WORKTREE_PATH}"
fi

docker compose -f "${MAIN_REPO_PATH}/compose.yaml" exec "${SERVICE}" \
  bash -c "cd ${WORK_DIR} && ${COMMAND}"

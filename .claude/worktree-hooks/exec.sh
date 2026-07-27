#!/bin/bash
# worktree-development スキルの exec.sh フック実装
# worktree 内で任意のコマンドを Docker Compose の backend コンテナ経由で実行する
#
# Usage: exec.sh <worktree_path> <command> [options...]
#   worktree_path: リポジトリルートから見た worktree のパス（例: .worktree/iddue/22）
#   command: 実行するコマンド
#
# compose.yaml の backend サービスに ./.worktree:/worktrees をマウントしているため、
# ホスト側の worktree_path はコンテナ内で /worktrees/{environment_name} として参照できる。

set -euo pipefail

WORKTREE_PATH="${1:?worktree_path を指定してください}"
COMMAND="${2:?command を指定してください}"

ENVIRONMENT_NAME="${WORKTREE_PATH#.worktree/}"
CONTAINER_BACKEND_DIR="/worktrees/${ENVIRONMENT_NAME}/backend"

docker compose exec -T backend bash -c "cd ${CONTAINER_BACKEND_DIR} && ${COMMAND}"

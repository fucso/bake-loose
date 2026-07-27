#!/bin/bash
# worktree-development スキルの exec.sh フック実装
# worktree 内で任意のコマンドを Docker Compose の backend コンテナ経由で実行する
#
# Usage: exec.sh <worktree_path> <command> [options...]
#   worktree_path: リポジトリルートから見た worktree のパス（例: .worktree/iddue/22）
#   command: 実行するコマンド（backend ディレクトリでの実行が必要な場合は "cd backend && ..." のように
#            コマンド側に含める。git 操作などリポジトリルートを対象とする場合はそのまま渡す）
#
# compose.yaml の backend サービスに ./.worktree:/worktrees をマウントしているため、
# ホスト側の worktree_path はコンテナ内で /worktrees/{environment_name} として参照できる。
# cd 先は worktree のルートまでに留め、対象ディレクトリ（backend/frontend等）の選択はコマンド側に委ねる。

set -euo pipefail

WORKTREE_PATH="${1:?worktree_path を指定してください}"
COMMAND="${2:?command を指定してください}"

ENVIRONMENT_NAME="${WORKTREE_PATH#.worktree/}"
CONTAINER_WORKTREE_DIR="/worktrees/${ENVIRONMENT_NAME}"

docker compose exec -T backend bash -c "cd ${CONTAINER_WORKTREE_DIR} && ${COMMAND}"

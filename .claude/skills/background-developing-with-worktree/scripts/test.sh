#!/bin/bash
# バックエンドのテストを実行
# 環境変数 MAIN_REPO_PATH, DOCKER_WORKTREE_PATH を使用
#
# 使用例: ./test.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
"${SCRIPT_DIR}/docker-exec.sh" backend "cargo test"

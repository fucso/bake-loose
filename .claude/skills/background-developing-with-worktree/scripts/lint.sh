#!/bin/bash
# フォーマットと Lint を実行
# 環境変数 MAIN_REPO_PATH, DOCKER_WORKTREE_PATH を使用
#
# 使用例: ./lint.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# フォーマット
"${SCRIPT_DIR}/docker-exec.sh" backend "cargo fmt"

# Lint
"${SCRIPT_DIR}/docker-exec.sh" backend "cargo clippy -- -D warnings"

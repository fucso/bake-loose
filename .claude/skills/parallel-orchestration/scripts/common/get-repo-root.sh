#!/bin/bash
# リポジトリルートのパスを出力する
# 使用例: REPO_ROOT=$("${SCRIPT_DIR}/common/get-repo-root.sh")

cd "$(dirname "$0")" && git rev-parse --show-toplevel

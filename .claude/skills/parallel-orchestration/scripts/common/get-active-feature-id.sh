#!/bin/bash
# active.yaml から進行中の feature-id を出力する
# 進行中の feature がない場合は空文字を出力
# 使用例: FEATURE_ID=$("${SCRIPT_DIR}/common/get-active-feature-id.sh")

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT=$("${SCRIPT_DIR}/get-repo-root.sh")
ACTIVE_YAML="${REPO_ROOT}/.agents/active.yaml"

if [[ ! -f "${ACTIVE_YAML}" ]]; then
  echo ""
  exit 0
fi

grep -A1 "active_feature:" "${ACTIVE_YAML}" 2>/dev/null \
  | grep "feature_id:" \
  | awk '{print $2}' \
  | tr -d '"' \
  || echo ""

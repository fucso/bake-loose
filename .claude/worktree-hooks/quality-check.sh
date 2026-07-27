#!/bin/bash
# worktree-development スキルの quality-check.sh フック実装
# worktree 内の backend コードに対して品質チェックを実行する
#
# Usage: quality-check.sh <worktree_path> <changed_files>
#   worktree_path: リポジトリルートから見た worktree のパス（例: .worktree/iddue/22）
#   changed_files: 変更ファイル一覧（未使用）
#
# cargo fmt --check → cargo clippy --all-targets -- -D warnings → cargo test の順に実行し、
# いずれかが失敗した時点で非ゼロ終了する。

set -euo pipefail

WORKTREE_PATH="${1:?worktree_path を指定してください}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

run() {
    bash "${SCRIPT_DIR}/exec.sh" "${WORKTREE_PATH}" "cd backend && $1"
}

echo "--- cargo fmt --check ---"
run "cargo fmt --check"

echo "--- cargo clippy --all-targets -- -D warnings ---"
run "cargo clippy --all-targets -- -D warnings"

echo "--- cargo test ---"
run "cargo test"

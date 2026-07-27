#!/usr/bin/env bash
set -euo pipefail

# Usage: fetch-review-comments.sh <owner> <repo> <pr_number> [review_id]
#
# gh API を使ってPRのレビューコメントを JSON で取得する。
# review_id が指定された場合はそのレビューのコメントのみに絞り込む。
#
# 出力: JSON Lines 形式（1行1コメント）

OWNER="${1:?Usage: $0 <owner> <repo> <pr_number> [review_id]}"
REPO="${2:?Usage: $0 <owner> <repo> <pr_number> [review_id]}"
PR_NUMBER="${3:?Usage: $0 <owner> <repo> <pr_number> [review_id]}"
REVIEW_ID="${4:-}"

JQ_FILTER='.[] | {
  id: .id,
  review_id: .pull_request_review_id,
  path: .path,
  line: .line,
  side: .side,
  body: .body,
  author: .user.login,
  created_at: .created_at,
  in_reply_to_id: .in_reply_to_id
}'

if [ -n "$REVIEW_ID" ]; then
  JQ_FILTER=".[] | select(.pull_request_review_id == ${REVIEW_ID}) | {
    id: .id,
    review_id: .pull_request_review_id,
    path: .path,
    line: .line,
    side: .side,
    body: .body,
    author: .user.login,
    created_at: .created_at,
    in_reply_to_id: .in_reply_to_id
  }"
fi

gh api "repos/${OWNER}/${REPO}/pulls/${PR_NUMBER}/comments" \
  --paginate \
  --jq "$JQ_FILTER"

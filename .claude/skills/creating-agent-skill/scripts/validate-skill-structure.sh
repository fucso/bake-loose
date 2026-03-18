#!/usr/bin/env bash
set -euo pipefail

# スキルディレクトリの構造を検証するスクリプト
# 使用例: bash .claude/skills/creating-agent-skill/scripts/validate-skill-structure.sh .claude/skills/{target-skill}/

SKILL_DIR="${1:?Usage: $0 <skill-directory>}"
SKILL_DIR="${SKILL_DIR%/}"  # 末尾スラッシュを除去

ERRORS=0
WARNINGS=0

error() {
  echo "❌ ERROR: $1"
  ERRORS=$((ERRORS + 1))
}

warn() {
  echo "⚠️  WARNING: $1"
  WARNINGS=$((WARNINGS + 1))
}

pass() {
  echo "✅ PASS: $1"
}

echo "=========================================="
echo "スキル構造検証: ${SKILL_DIR}"
echo "=========================================="
echo ""

# --- A1: SKILL.md の存在チェック ---
if [ -f "${SKILL_DIR}/SKILL.md" ]; then
  pass "A1: SKILL.md が存在する"
else
  error "A1: SKILL.md が存在しない: ${SKILL_DIR}/SKILL.md"
  echo ""
  echo "結果: SKILL.md が見つからないため検証を中断します"
  exit 1
fi

SKILL_MD="${SKILL_DIR}/SKILL.md"

# --- A2: frontmatter に name が存在 ---
if head -20 "${SKILL_MD}" | grep -q "^name:"; then
  pass "A2: frontmatter に name が存在"
else
  error "A2: frontmatter に name が存在しない"
fi

# --- A3: frontmatter に description が存在 ---
if head -20 "${SKILL_MD}" | grep -q "^description:"; then
  pass "A3: frontmatter に description が存在"
else
  error "A3: frontmatter に description が存在しない"
fi

# --- A4: name の命名規則チェック ---
NAME=$(head -20 "${SKILL_MD}" | grep "^name:" | head -1 | sed 's/^name:[[:space:]]*//')
if [ -n "${NAME}" ]; then
  if echo "${NAME}" | grep -qE "^[a-z0-9][a-z0-9-]*$"; then
    if [ ${#NAME} -le 64 ]; then
      pass "A4: name '${NAME}' は命名規則に準拠"
    else
      error "A4: name '${NAME}' が64文字を超えている (${#NAME}文字)"
    fi
  else
    error "A4: name '${NAME}' が命名規則に違反 (小文字・数字・ハイフンのみ許可)"
  fi
else
  warn "A4: name が空のためチェックをスキップ"
fi

# --- A5: SKILL.md の行数チェック ---
LINE_COUNT=$(wc -l < "${SKILL_MD}" | tr -d ' ')
if [ "${LINE_COUNT}" -le 100 ]; then
  pass "A5: SKILL.md は ${LINE_COUNT} 行 (≤100)"
elif [ "${LINE_COUNT}" -le 150 ]; then
  warn "A5: SKILL.md は ${LINE_COUNT} 行 (>100, ≤150)"
else
  error "A5: SKILL.md は ${LINE_COUNT} 行 (>150)"
fi

# --- A6: 参照が1階層のみか ---
if [ -d "${SKILL_DIR}/references" ]; then
  # マークダウンリンク形式 [text](references/...) で他の references を参照しているかチェック
  NESTED_REFS=$(grep -rlE '\]\(references/' "${SKILL_DIR}/references/" 2>/dev/null | head -5 || true)
  if [ -z "${NESTED_REFS}" ]; then
    pass "A6: 参照ファイルは1階層のみ"
  else
    error "A6: references/ 内で他の references/ を参照しているファイルあり:"
    echo "${NESTED_REFS}" | while read -r f; do
      echo "      - ${f}"
    done
  fi
else
  pass "A6: references/ ディレクトリなし (チェック不要)"
fi

# --- A7: Windows パスの検出 ---
# バックスラッシュ + ドライブレター or ディレクトリ名パターン (e.g., C:\Users, .\path)
WIN_PATHS=$(grep -rnP '[A-Z]:\\\\|\\\\[A-Za-z]+\\\\' "${SKILL_DIR}/"*.md "${SKILL_DIR}/references/"*.md "${SKILL_DIR}/templates/"*.md 2>/dev/null | head -5 || true)
if [ -z "${WIN_PATHS}" ]; then
  pass "A7: Windows パスは検出されず"
else
  warn "A7: Windows パスの可能性あり (誤検出の場合あり):"
  echo "${WIN_PATHS}" | while read -r line; do
    echo "      ${line}"
  done
fi

# --- 結果サマリー ---
echo ""
echo "=========================================="
echo "結果サマリー"
echo "=========================================="
echo "  PASS:    $((7 - ERRORS - WARNINGS))"
echo "  WARNING: ${WARNINGS}"
echo "  ERROR:   ${ERRORS}"
echo ""

if [ "${ERRORS}" -gt 0 ]; then
  echo "❌ FAIL: ${ERRORS} 件のエラーがあります"
  exit 1
elif [ "${WARNINGS}" -gt 0 ]; then
  echo "⚠️  WARNING: ${WARNINGS} 件の警告があります"
  exit 0
else
  echo "✅ ALL PASS: すべてのチェックに合格しました"
  exit 0
fi

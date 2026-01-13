---
name: create-pr
description: |
  git コマンドと GitHub REST API を使用して、現在のブランチの変更を分析し、GitHub への push と PR 作成を行う。
  既存 PR がある場合は内容をチェックし、変更内容との差異があれば修正を提案する。
---

# Create PR

Claude Code on the Web 環境で git push と PR 作成を行う。

## 環境の制約

Claude Code on the Web では以下の制約がある：

- ローカルプロキシ経由の git push が 403 エラーで失敗する
- `gh` CLI が使用できない（GitHub Releases からのダウンロードがブロックされる）
- `GITHUB_TOKEN` 環境変数が利用可能

## ワークフロー

### 1. 変更の確認

```bash
git status
git log --oneline -5
git diff --stat HEAD~1
```

### 2. リモートURLの設定

push 前に、トークン認証付きの GitHub URL に変更する：

```bash
git remote set-url origin https://${GITHUB_TOKEN}@github.com/OWNER/REPO.git
```

### 3. Push

```bash
git push -u origin BRANCH_NAME
```

### 4. 既存PRの確認

curl で既存 PR を検索：

```bash
curl -s -H "Authorization: token ${GITHUB_TOKEN}" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/OWNER/REPO/pulls?head=OWNER:BRANCH_NAME&state=open"
```

### 5. PR 作成または更新

**新規作成の場合：**

```bash
curl -s -X POST \
  -H "Authorization: token ${GITHUB_TOKEN}" \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/OWNER/REPO/pulls \
  -d '{
    "title": "PR タイトル",
    "head": "BRANCH_NAME",
    "base": "main",
    "body": "## Summary\n- 変更点1\n- 変更点2\n\n## Test plan\n- [ ] テスト項目\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)"
  }'
```

**既存PRの更新が必要な場合：**

```bash
curl -s -X PATCH \
  -H "Authorization: token ${GITHUB_TOKEN}" \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/OWNER/REPO/pulls/PR_NUMBER \
  -d '{
    "title": "更新後のタイトル",
    "body": "更新後の本文"
  }'
```

## トラブルシューティング

### push が 403 で失敗する

リモートURLがトークン付きになっているか確認：

```bash
git remote -v
# https://${GITHUB_TOKEN}@github.com/... の形式であること
```

### GITHUB_TOKEN が未設定

環境変数を確認：

```bash
echo $GITHUB_TOKEN | head -c 10
# ghp_ または ghs_ で始まるトークンが表示されるはず
```

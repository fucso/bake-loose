---
name: create-pr
description: |
  git コマンドと GitHub MCP を使用して、現在のブランチの変更を分析し、GitHub への push と PR 作成を行う。
  既存 PR がある場合は内容をチェックし、変更内容との差異があれば修正を提案する。
tools:
  - Bash
  - mcp__github__search_pull_requests
  - mcp__github__pull_request_read
  - mcp__github__create_pull_request
  - mcp__github__update_pull_request
---

# 処理フロー

### Step 1: 変更内容の分析

#### 1.1 ブランチ情報の取得

```bash
# 現在のブランチ名
git branch --show-current

# 上流（追跡）ブランチの確認
git rev-parse --abbrev-ref --symbolic-full-name @{upstream} 2>/dev/null
```

#### 1.2 ベースブランチの特定

`git show-branch` を使用して、現在のブランチの派生元（ベースブランチ）を自動特定する。

```bash
# ベースブランチを特定
git show-branch | grep '*' | grep -v "$(git rev-parse --abbrev-ref HEAD)" | head -1 | awk -F'[]~^[]' '{print $2}'
```

**フォールバック:**

上記コマンドで特定できない場合（出力が空の場合）は、リポジトリのデフォルトブランチを使用:

```bash
git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main"
```

**例:**
- `feature/auth` から `feature/auth-login` を切った場合 → `feature/auth` が特定される
- `main` から直接切った場合 → `main` が特定される

#### 1.3 コミット履歴の取得

```bash
# 分岐点からのコミットログ（<base> は 1.2 で特定したベースブランチ）
git log --oneline origin/<base>..HEAD

# 詳細なコミットメッセージ
git log --format="%h %s%n%n%b" origin/<base>..HEAD
```

#### 1.4 変更差分の取得

```bash
# 変更ファイル一覧と統計
git diff --stat origin/<base>..HEAD

# 変更の詳細（コード内容の理解に必要な場合）
git diff origin/<base>..HEAD
```

#### 1.5 変更内容の分析

取得した情報から以下を特定:

1. **変更の種類**: 新機能 / バグ修正 / リファクタリング / ドキュメント / 設定変更 / テスト
2. **影響範囲**: 変更されたモジュール、レイヤー
3. **要件レベルの変更内容**: 「何を実現したか」の観点で整理

---

### Step 2: リモートへの Push

#### 2.1 リモート状況の確認

```bash
# 現在のブランチのリモート追跡状況
git status -sb

# 未 push のコミット確認
git log --oneline origin/$(git branch --show-current)..HEAD 2>/dev/null || echo "リモートブランチなし"
```

#### 2.2 Push 実行（未 push のコミットがある場合）

```bash
# upstream を設定して push
git push -u origin $(git branch --show-current)
```

**エラー時の対処:**
- リジェクト → `git pull --rebase origin <branch>` を提案
- 権限エラー → リポジトリへのアクセス権確認を促す

---

### Step 3: PR の作成

#### 3.1 リポジトリ情報の取得

```bash
# リモート URL からオーナーとリポジトリ名を取得
git remote get-url origin
```

URL パターン:
- `git@github.com:owner/repo.git` → owner, repo
- `https://github.com/owner/repo.git` → owner, repo

#### 3.2 既存 PR の確認

GitHub MCP `search_pull_requests` を使用:

```
query: "head:<current-branch> is:open"
owner: <owner>
repo: <repo>
```

**既存 PR がある場合:** Step 3.2.1 へ進む
**既存 PR がない場合:** Step 3.3 へ進む

#### 3.2.1 既存 PR の内容チェック（PR が存在する場合）

GitHub MCP `mcp__github__pull_request_read` で既存 PR の詳細を取得:

```
method: "get"
owner: <owner>
repo: <repo>
pullNumber: <pr_number>
```

**Step 1 の分析結果と比較して以下をチェック:**

1. **タイトルの妥当性**
   - 変更内容を適切に表現しているか
   - 要件ベースで記述されているか

2. **説明文の網羅性**
   - 概要セクション: 変更の目的が記載されているか
   - 対応内容セクション: 実際の変更が漏れなく記載されているか
   - ポイントセクション: 重要な変更が記載されているか
   - 申し送りセクション: 必要に応じて記載されているか

3. **差分の検出**
   - Step 1 で分析した変更内容と PR 説明文の差異を特定
   - 新しいコミットによる変更が反映されているか

**差異がある場合:**

修正案を作成し、以下の形式で報告:

```
既存の PR を確認しました: <PR URL>

【現在のタイトル】
<current_title>

【現在の説明文】
<current_body>

【分析結果との差異】
- <差異1>
- <差異2>

【修正提案】

タイトル（変更が必要な場合）:
<suggested_title>

説明文:
<suggested_body>

この修正を適用しますか？
```

**修正の適用:**

ユーザーが承認した場合、GitHub MCP `mcp__github__update_pull_request` を使用:

```
owner: <owner>
repo: <repo>
pullNumber: <pr_number>
title: <suggested_title>  # 変更がある場合
body: <suggested_body>
```

**差異がない場合:**

PR の URL と現在の状態を報告して終了。

#### 3.3 PR 説明文の作成（新規作成の場合）

Step 1 の分析結果を基に以下の形式で作成:

```markdown
## 概要

[PR で行なったことを簡潔に記載]
[実装ベースではなく要件・目的ベースで記述]

## 対応内容

[概要の要件を実現するために行なったことを箇条書きで]

- 対応項目1
- 対応項目2
- 対応項目3

## ポイント

[重要な変更や注意すべき点があれば記載]
[なければこのセクションは省略]

## 申し送り

[本来は PR 内で対応すべきだが申し送りとしていること]
[なければこのセクションは省略]
```

#### 3.4 PR の作成

GitHub MCP `mcp__github__create_pull_request` を使用:

```
owner: <owner>
repo: <repo>
title: <分析結果から適切なタイトル>
head: <current-branch>
base: <Step 1.2 で特定したベースブランチ>
body: <上記で作成した説明文>
```

---

### Step 4: 結果報告

以下を報告:

1. **作成した PR の URL**
2. **PR タイトル**
3. **変更の概要**

---

## PR 説明文のガイドライン

### 概要セクション
- 1-3 文で簡潔に
- 「XXX を実装しました」ではなく「XXX できるようになりました」
- 技術用語より機能・目的を優先

### 対応内容セクション
- 箇条書きで列挙
- 各項目は動詞で開始（追加、修正、削除、更新など）
- 具体的なファイル名やコンポーネント名を含める

### ポイントセクション（任意）
- 破壊的変更がある場合は必ず記載
- レビュー時に注目してほしい点
- 設計上の判断理由

### 申し送りセクション（任意）
- TODO として残した項目
- 後続 PR で対応予定の項目
- 既知の制限事項

---

## エラーハンドリング

| 状況 | 対処 |
|------|------|
| push 失敗（認証） | ユーザーに認証設定の確認を促す |
| push 失敗（リジェクト） | rebase を提案 |
| PR 作成失敗（既存） | 既存 PR の URL を報告 |
| PR 作成失敗（権限） | フォークからの PR 作成を提案 |
| ブランチが main | PR 作成不可を報告、新ブランチ作成を提案 |

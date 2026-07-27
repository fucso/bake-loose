# ワークフロー

PR レビューコメントを取得・構造化するための実行手順。

---

## 目次

1. [ワークフロー概要](#ワークフロー概要)
2. [Step 1: URL パース](#step-1-url-パース)
3. [Step 2: データ取得](#step-2-データ取得)
4. [Step 3: フィルタリング](#step-3-フィルタリング)
5. [Step 4: 構造化出力](#step-4-構造化出力)
6. [エラーハンドリング](#エラーハンドリング)

---

## ワークフロー概要

| Step | 主体 | 内容 |
|------|------|------|
| 1. URL パース | 🤖 Claude 自動 | URL から owner, repo, number, review_id を抽出 |
| 2. データ取得 | 🤖 Claude 自動 | gh コマンドで3種類のコメントを並列取得 |
| 3. フィルタリング | 🤖 Claude 自動 | review_id 指定時は該当レビューのみに絞る |
| 4. 構造化出力 | 🤖 Claude 自動 | ファイル別・行番号順にグルーピングして出力 |

---

## Step 1: URL パース

入力 URL から以下の情報を抽出する。

### 対応フォーマット

| 形式 | パターン |
|------|---------|
| PR URL | `https://github.com/{owner}/{repo}/pull/{number}` |
| レビュー URL | `https://github.com/{owner}/{repo}/pull/{number}#pullrequestreview-{review_id}` |
| コメント URL | `https://github.com/{owner}/{repo}/pull/{number}#issuecomment-{comment_id}` |

### 抽出する値

| 値 | 必須 | 説明 |
|----|------|------|
| `owner` | はい | リポジトリオーナー |
| `repo` | はい | リポジトリ名 |
| `number` | はい | PR 番号 |
| `review_id` | いいえ | 特定レビューの ID（フィルタリングに使用） |

URL が解析できない場合は、エラーハンドリングセクションを参照。

---

## Step 2: データ取得

以下の3つのデータを `gh` コマンドで取得する。**独立したコマンドなので並列実行する。**

### 2.1 インラインレビューコメント

```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  --jq '.[] | {
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
```

### 2.2 PR 本体コメント（非インライン）

```bash
gh api repos/{owner}/{repo}/issues/{number}/comments \
  --jq '.[] | {
    id: .id,
    body: .body,
    author: .user.login,
    created_at: .created_at
  }'
```

### 2.3 レビューサマリー

```bash
gh api repos/{owner}/{repo}/pulls/{number}/reviews \
  --jq '.[] | {
    id: .id,
    state: .state,
    body: .body,
    author: .user.login,
    submitted_at: .submitted_at
  }'
```

### ページネーション

コメントが 30 件を超える場合、`--paginate` オプションを付与して全件取得する。

```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments --paginate --jq '...'
```

---

## Step 3: フィルタリング

| 条件 | アクション |
|------|----------|
| `review_id` が指定されている | インラインコメントを `review_id` フィールドで絞り込み、レビューサマリーを `id` フィールドで絞り込み |
| `review_id` が未指定 | すべてのコメントを出力 |

---

## Step 4: 構造化出力

### 出力フォーマット

```markdown
## PR #{number}: {title}

- **状態**: {state}
- **ブランチ**: {headRefName} → {baseRefName}
- **作成者**: {author}
```

### レビューサマリー

body が空でないレビューのみ表示する。

```markdown
### レビューサマリー

> **{author}** ({state}) - {submitted_at}
>
> {body}
```

### インラインコメント

**ファイルごとにグルーピング**し、**行番号の昇順**にソートして表示する。

```markdown
### インラインコメント

#### `{path}`

| 行 | コメント | 投稿者 |
|----|----------|--------|
| {line} | {body} | {author} |
```

### スレッドの処理

`in_reply_to_id` が設定されているコメントは返信として扱い、親コメントの直後に表示する。

```markdown
| {line} | {parent body} | {parent author} |
|        | ↳ {reply body} | {reply author} |
```

### PR コメント

非インラインの PR コメントがある場合のみ表示する。

```markdown
### PR コメント

> **{author}** - {created_at}
>
> {body}
```

### 表示上の注意

- コメント本体の改行はテーブル表示では半角スペースに置換する
- コメント本体の `|` は `\|` にエスケープする
- 全件出力する（件数が多くても省略しない）

---

## エラーハンドリング

| エラー | 対処 |
|--------|------|
| URL が解析できない | 対応フォーマットを提示して再入力を促す |
| `gh` 認証エラー (401) | `gh auth login` の実行を案内 |
| リポジトリが見つからない (404) | URL の owner/repo が正しいか確認を促す |
| PR が見つからない (404) | PR 番号が正しいか確認を促す |
| コメントが 0 件 | 「コメントはありません」と報告 |

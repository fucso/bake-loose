# report.md テンプレート

ワーカーがタスク完了時に作成するレポート。オーケストレーターは git を監視してこのファイルのコミットで完了を検知する。

## 配置場所

worktree 内のタスクディレクトリに作成:

```
{worktree_path}/.agents/features/{feature-id}/tasks/{task-id}/report.md
```

---

## 基本構造

```markdown
# Task Report: {タスク名}

> 実施日時: {YYYY-MM-DD HH:MM}
> 依存タスク: {依存タスク名（あれば、なければ「なし」）}

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| {パス} | 新規/修正/削除 | {概要} |

## ビルド・テスト結果

### コンパイル/ビルド

{結果}

### テスト

{結果}

## コミット情報

- ハッシュ: {commit_hash}
- ブランチ: {branch_name}

## 次タスクへの申し送り

{後続タスク実装者が知っておくべき情報}
```

## 例（成功）

```markdown
# Task Report: スキル定義

> 実施日時: 2026-02-09 11:30
> 依存タスク: なし

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| .claude/skills/parallel-orchestration/SKILL.md | 新規 | 機構の説明とガイドライン |
| .claude/skills/parallel-orchestration/references/file-formats.md | 新規 | ファイルフォーマット仕様 |
| .claude/skills/parallel-orchestration/references/commands.md | 新規 | コマンド詳細 |
| .claude/skills/parallel-orchestration/references/worker-behavior.md | 新規 | ワーカーの振る舞い |
| .claude/skills/parallel-orchestration/references/orchestrator-behavior.md | 新規 | オーケストレーターの振る舞い |
| .claude/skills/parallel-orchestration/references/troubleshooting.md | 新規 | トラブルシューティング |
| .claude/skills/parallel-orchestration/appendix/active-yaml.md | 新規 | テンプレート |
| .claude/skills/parallel-orchestration/appendix/tasks-yaml.md | 新規 | テンプレート |
| .claude/skills/parallel-orchestration/appendix/status-yaml.md | 新規 | テンプレート |
| .claude/skills/parallel-orchestration/appendix/report-md.md | 新規 | テンプレート |

## ビルド・テスト結果

### コンパイル/ビルド

該当なし（ドキュメントのみ）

### テスト

該当なし

## コミット情報

- ハッシュ: abc1234
- ブランチ: task/20260209-parallel_orchestration_01-skill-definition

## 次タスクへの申し送り

- ワーカーの振る舞いは `references/worker-behavior.md` に記載しています
- ワークフロー図は `references/workflow.md` を参照してください
```

## 例（エラー発生）

```markdown
# Task Report: 統合テスト

> 実施日時: 2026-02-09 16:00
> 依存タスク: 02-design-command, 03-worker-agent, 04-orchestrator-start

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| tests/orchestration/integration_test.rs | 新規 | 統合テスト |

## ビルド・テスト結果

### コンパイル/ビルド

成功

### テスト

**失敗**

```
test orchestration::test_parallel_execution ... FAILED

thread 'orchestration::test_parallel_execution' panicked at 'assertion failed: `(left == right)`
  left: `"completed"`,
 right: `"in_progress"`, tests/orchestration/integration_test.rs:45:5
```

## コミット情報

- ハッシュ: ghi9012
- ブランチ: task/20260209-parallel_orchestration_05-integration

## 次タスクへの申し送り

- 統合テストが失敗しています
- status.yaml の更新タイミングに問題がある可能性があります
- 手動でのデバッグが必要です
```

## セクション説明

| セクション | 必須 | 説明 |
|-----------|------|------|
| タイトル | ○ | タスク名を含める |
| メタ情報 | ○ | 実施日時、依存タスク |
| 変更ファイル | ○ | 変更したファイルの一覧 |
| ビルド・テスト結果 | ○ | 該当なし含め記載 |
| コミット情報 | ○ | ハッシュとブランチ名 |
| 次タスクへの申し送り | ○ | 後続タスクへの情報共有 |

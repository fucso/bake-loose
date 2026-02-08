# Task: ワーカーエージェント

> Feature: [parallel_orchestration](../../spec.md)
> 依存: 01-skill-definition

## 目的

オーケストレーターから起動され、単一タスクを自律的に実行するワーカーエージェントを定義する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.claude/agents/parallel-worker.md` | 新規 | ワーカーエージェント定義 |

---

## 設計詳細

### エージェント概要

- オーケストレーターが `claude -p` で起動
- 指定されたタスクを自律的に実行
- 完了時に `report.md` を作成

### 起動方法

```bash
cd {worktree_path} && claude -p "{prompt}"
```

ワーカーは worktree 内で起動されるため、自身が worktree を使っていることを意識しない。
現在のディレクトリがプロジェクトルートとして振る舞う。

### ワーカー用プロンプト（初期版）

```
以下のタスクを実行してください。

## タスク情報
- タスク仕様: {task_spec_path}
- Feature 仕様: {feature_spec_path}

## 実行手順
1. タスク仕様（spec.md）を読み込む
2. 依存タスクの report.md があれば参照する
3. 対象レイヤーのコーディングルールを参照する
4. 実装を行う
5. ビルド・テストを実行する
6. 変更をコミットする
7. report.md を作成する

## 制約
- report.md 以外の .agents/ 配下のファイルは編集しない
- タスク仕様に記載された範囲のみを実装する
- Docker Compose 環境は共有されている

## 完了条件
タスクディレクトリに report.md を作成することで完了を通知する。
```

### ワーカーの制約

**編集可能:**
- タスク仕様で指定されたソースコード
- `{task_dir}/report.md`

**編集禁止:**
- `active.yaml`
- `tasks.yaml`
- `status.yaml`
- 他タスクの `spec.md` / `report.md`

### report.md のフォーマット

```markdown
# Task Report: {タスク名}

> 実施日時: {YYYY-MM-DD HH:MM}
> 依存タスク: {依存タスク名（あれば）}

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| {パス} | 新規/修正 | {概要} |

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

---

## 参照すべきドキュメント

- `.claude/skills/parallel-orchestration/SKILL.md`
- `.claude/commands/dev.md`（既存の実装コマンド、参考）

---

## 完了条件

- [ ] `.claude/agents/parallel-worker.md` が作成されている
- [ ] ワーカー用プロンプトが定義されている
- [ ] 制約事項が明記されている
- [ ] `report.md` のフォーマットが定義されている

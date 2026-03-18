# Task: 状況確認コマンド

> Feature: [parallel_orchestration](../../spec.md)
> 依存: 01-skill-definition

## 目的

並列オーケストレーションの進捗状況を確認するコマンド `/orchestrate:status` を作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.claude/commands/orchestrate/status.md` | 新規 | 状況確認コマンド |

---

## 設計詳細

### コマンド概要

```
/orchestrate:status [feature-id]
```

- `feature-id` 省略時: `.agents/active.yaml` から現在進行中の Feature を取得
- `feature-id` 指定時: 指定された Feature の状況を表示

### 出力フォーマット

```
## 並列オーケストレーション状況

**Feature:** {feature-id}
**ステータス:** {status}
**開始日時:** {started_at}
**最終更新:** {updated_at}

### 進捗サマリー

| 状態 | タスク数 |
|------|----------|
| 完了 | {n} |
| 実行中 | {n} |
| 待機中 | {n} |
| 合計 | {n} |

### 実行中タスク

| タスク | 開始日時 | 経過時間 |
|--------|----------|----------|
| {task_id} | {started_at} | {elapsed} |

### 完了済みタスク

- {task_id_1}
- {task_id_2}
- ...

### 待機中タスク

| タスク | ブロック要因 |
|--------|-------------|
| {task_id} | {blocking_tasks} |
```

### 処理フロー

1. `feature-id` が指定されていない場合、`.agents/active.yaml` を読み込む
   - `active_feature` が null なら「進行中の Feature はありません」と表示
2. `status.yaml` を読み込む
3. `tasks.yaml` を読み込み（依存関係の表示用）
4. 上記フォーマットで出力

### ブロック要因の計算

待機中タスクの「ブロック要因」は以下のロジックで計算:

1. `tasks.yaml` から該当タスクの `dependencies` を取得
2. `status.yaml` の `completed_tasks` に含まれていない依存を抽出
3. 抽出された依存タスクIDをカンマ区切りで表示

### エラーハンドリング

| エラー | 対処 |
|--------|------|
| Feature が見つからない | エラー、利用可能な Feature 一覧を表示 |
| `status.yaml` が見つからない | 「まだ開始されていません」と表示 |

---

## 完了条件

- [ ] `.claude/commands/orchestrate/status.md` が作成されている
- [ ] 出力フォーマットが定義されている
- [ ] ブロック要因の計算ロジックが定義されている
- [ ] エラーハンドリングが定義されている

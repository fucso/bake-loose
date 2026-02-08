# Task: オーケストレーター起動

> Feature: [parallel_orchestration](../../spec.md)
> 依存: 01-skill-definition, 03-worker-agent

## 目的

Feature の並列タスク実行を開始するオーケストレーターコマンド `/orchestrate:start` を作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.claude/commands/orchestrate/start.md` | 新規 | オーケストレーター起動コマンド |

---

## 設計詳細

### コマンド概要

```
/orchestrate:start {feature-id}
```

### 処理フロー

#### Phase 1: 初期化

1. `.agents/active.yaml` を確認
   - 既に他の Feature が進行中なら警告して中断
2. `tasks.yaml` を読み込み、依存グラフを構築
3. Feature ブランチを作成（`feature/{feature-id}`）
4. `.agents/active.yaml` を更新
5. `status.yaml` を作成（初期状態）

#### Phase 2: 依存解決とディスパッチ

1. 依存関係が解消されたタスク（unblocked）を抽出
   - `tasks.yaml` の `dependencies` が空、または全て `completed_tasks` に含まれる
2. 各 unblocked タスクに対して:
   a. git worktree を作成
      ```bash
      git worktree add .agents/worktrees/{feature-id}_{task-id} -b task/{feature-id}_{task-id}
      ```
   b. ワーカープロセスを起動
      ```bash
      cd {worktree_path} && claude -p "{prompt}" > {task_dir}/worker_output.log 2>&1 &
      ```
   c. `status.yaml` を更新（`active_tasks` に追加、`pending_tasks` から削除）

#### Phase 3: 監視ループ

1. `active_tasks` が空になるまでループ
2. 各タスクの `report.md` を監視（ポーリング）
3. `report.md` が作成されたら:
   a. 内容を読み取り、成功/失敗を判定
   b. `status.yaml` を更新
      - `active_tasks` から削除
      - `completed_tasks` に追加
   c. worktree を削除（オプション、または cleanup で行う）
   d. `tasks.yaml` を参照して依存解決
   e. 新たに unblocked になったタスクをディスパッチ（Phase 2 に戻る）

#### Phase 4: 完了判定

1. 全タスクが `completed_tasks` に含まれたら完了
2. `status.yaml` を更新（`status: completed`）
3. `.agents/active.yaml` をクリア
4. 完了レポートを出力

### worktree の命名規則

```
.agents/worktrees/{feature-id}_{task-id}/
```

例: `.agents/worktrees/20260209-parallel_orchestration_01-skill-definition/`

### ブランチの命名規則

```
task/{feature-id}_{task-id}
```

例: `task/20260209-parallel_orchestration_01-skill-definition`

### エラーハンドリング

| エラー | 対処 |
|--------|------|
| 既に他 Feature が進行中 | 警告して中断、`/orchestrate:status` を案内 |
| `tasks.yaml` が見つからない | エラー、`/orchestrate:design` を案内 |
| ワーカーがクラッシュ | `status.yaml` に失敗を記録、ユーザーに報告 |
| 循環依存の検出 | エラー、`tasks.yaml` の修正を案内 |

---

## 参照すべきドキュメント

- `.claude/skills/parallel-orchestration/SKILL.md`
- `.claude/agents/parallel-worker.md`

---

## 完了条件

- [ ] `.claude/commands/orchestrate/start.md` が作成されている
- [ ] 初期化フェーズが定義されている
- [ ] 依存解決とディスパッチのロジックが定義されている
- [ ] 監視ループが定義されている
- [ ] エラーハンドリングが定義されている

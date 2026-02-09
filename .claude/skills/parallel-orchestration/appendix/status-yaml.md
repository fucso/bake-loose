# status.yaml テンプレート

実行状態を管理する動的ファイル。オーケストレーターのみが更新する。

## 配置場所

```
.agents/features/{feature-id}/status.yaml
```

## ステータス遷移

```
pending → in_progress → completed
                     ↘ failed
```

| ステータス | 説明 |
|-----------|------|
| `pending` | 初期状態（まだ開始されていない） |
| `in_progress` | 実行中 |
| `completed` | 全タスク完了 |
| `failed` | 1つ以上のタスクが失敗 |

---

## 初期状態

```yaml
status: pending
feature_branch: feature/{feature-id}
started_at: {ISO 8601 datetime}
updated_at: {ISO 8601 datetime}

active_tasks: []

completed_tasks: []

pending_tasks:
  - {task-id-1}
  - {task-id-2}
  - ...
```

## 実行中

```yaml
status: in_progress
feature_branch: feature/{feature-id}
started_at: {ISO 8601 datetime}
updated_at: {ISO 8601 datetime}

active_tasks:
  - task_id: {task-id}
    worktree_path: .agents/worktrees/{feature-id}_{task-id}
    branch: task/{feature-id}_{task-id}
    worker_pid: {pid}
    started_at: {ISO 8601 datetime}

completed_tasks:
  - {completed-task-id}

pending_tasks:
  - {pending-task-id}
```

## 完了

```yaml
status: completed
feature_branch: feature/{feature-id}
started_at: {ISO 8601 datetime}
updated_at: {ISO 8601 datetime}

active_tasks: []

completed_tasks:
  - {task-id-1}
  - {task-id-2}
  - ...

pending_tasks: []
```

## 失敗

```yaml
status: failed
feature_branch: feature/{feature-id}
started_at: {ISO 8601 datetime}
updated_at: {ISO 8601 datetime}

active_tasks: []

completed_tasks:
  - {completed-task-id}

pending_tasks:
  - {pending-task-id}

failed_task: {failed-task-id}
error_message: {エラー内容}
```

## 例（実行中）

```yaml
status: in_progress
feature_branch: feature/20260209-parallel_orchestration
started_at: 2026-02-09T10:30:00+09:00
updated_at: 2026-02-09T11:15:00+09:00

active_tasks:
  - task_id: 02-design-command
    worktree_path: .agents/worktrees/20260209-parallel_orchestration_02-design-command
    branch: task/20260209-parallel_orchestration_02-design-command
    worker_pid: 12346
    started_at: 2026-02-09T11:00:00+09:00

  - task_id: 03-worker-agent
    worktree_path: .agents/worktrees/20260209-parallel_orchestration_03-worker-agent
    branch: task/20260209-parallel_orchestration_03-worker-agent
    worker_pid: 12347
    started_at: 2026-02-09T11:00:00+09:00

completed_tasks:
  - 01-skill-definition

pending_tasks:
  - 04-orchestrator-start
  - 05-integration
```

## フィールド説明

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `status` | string | `pending` / `in_progress` / `completed` / `failed` |
| `feature_branch` | string | Feature ブランチ名 |
| `started_at` | datetime | 開始日時（ISO 8601） |
| `updated_at` | datetime | 最終更新日時（ISO 8601） |
| `active_tasks` | array | 現在実行中のタスク |
| `active_tasks[].task_id` | string | タスクID |
| `active_tasks[].worktree_path` | string | worktree のパス |
| `active_tasks[].branch` | string | タスクブランチ名 |
| `active_tasks[].worker_pid` | int | ワーカーのプロセスID |
| `active_tasks[].started_at` | datetime | タスク開始日時 |
| `completed_tasks` | array | 完了済みタスクIDのリスト |
| `pending_tasks` | array | 待機中タスクIDのリスト |
| `failed_task` | string | 失敗したタスクID（失敗時のみ） |
| `error_message` | string | エラー内容（失敗時のみ） |

---
name: parallel-orchestration
description: |
  複数タスクの並列オーケストレーション機構。オーケストレーターがワーカーを統制し、ファイルベースで状態を管理することで、Feature 内のタスクを効率的に並列実行する。
  使用タイミング: (1) /orchestrate:design での設計、(2) /orchestrate:start での並列実行、(3) ワーカーとしてのタスク実行
user-invocable: false
---

# 並列オーケストレーション

Feature 内のタスクを並列実行するための機構。

## 概要

オーケストレーターとワーカーの役割分担により、依存関係を考慮しながら複数タスクを効率的に並列実行する。

### 役割分担

| 役割 | 責務 | 編集可能ファイル |
|------|------|-----------------|
| **オーケストレーター** | worktree 準備、ワーカー起動、状態管理 | `active.yaml`, `status.yaml`, worktree 操作 |
| **ワーカー** | タスク実行、レポート作成 | ソースコード、`report.md` のみ |

### ワークフロー図

```
[設計] /orchestrate:design
    ↓
[実行] /orchestrate:start
    ↓
[監視] status.yaml + report.md
    ↓
[完了] 全タスク完了時に自動処理
```

詳細なシーケンス図と依存解決の例は [references/workflow.md](references/workflow.md) を参照。

---

## コマンド一覧

| コマンド | 概要 | 実装・詳細 |
|----------|------|-----------|
| `/orchestrate:design` | 並列実行用の設計 | [workflow.md Phase 1](references/workflow.md) |
| `/orchestrate:start` | オーケストレーター起動 | [orchestrator-behavior.md](references/orchestrator-behavior.md) |

---

## ファイル構成

```
.agents/
├── active.yaml            # 現在進行中の Feature
├── worktrees/             # git worktree の実体
└── features/{feature-id}/
    ├── spec.md            # Feature 仕様
    ├── tasks.yaml         # タスク定義・依存関係
    ├── status.yaml        # 実行状態
    └── tasks/{task-name}/
        ├── spec.md        # タスク仕様
        └── report.md      # ワーカーが作成
```

## ファイルフォーマット

| ファイル | 用途 | 更新者 | テンプレート |
|----------|------|--------|--------------|
| `active.yaml` | 進行中 Feature の管理 | オーケストレーター | [appendix/active-yaml.md](appendix/active-yaml.md) |
| `tasks.yaml` | タスク定義（静的） | `/orchestrate:design` | [appendix/tasks-yaml.md](appendix/tasks-yaml.md) |
| `status.yaml` | 実行状態（動的） | オーケストレーター | [appendix/status-yaml.md](appendix/status-yaml.md) |
| `report.md` | タスク完了報告 | ワーカー | [appendix/report-md.md](appendix/report-md.md) |

---

## 役割別ガイド

| 役割 | ガイド |
|------|--------|
| オーケストレーター | [references/orchestrator-behavior.md](references/orchestrator-behavior.md) |
| ワーカー | [references/worker-behavior.md](references/worker-behavior.md) |

---

## トラブルシューティング

よくある問題と対処法は [references/troubleshooting.md](references/troubleshooting.md) を参照。

---


# Task: スキル定義

> Feature: [parallel_orchestration](../../spec.md)
> 依存: なし

## 目的

並列オーケストレーション機構の中核となるスキルを定義する。
このスキルは機構全体の説明、ファイルフォーマット仕様、ワークフローガイドラインを提供する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.claude/skills/parallel-orchestration/SKILL.md` | 新規 | 機構の説明とガイドライン |

---

## 設計詳細

### SKILL.md の構成

1. **概要**
   - 並列オーケストレーション機構の目的と全体像
   - オーケストレーターとワーカーの役割分担

2. **ファイルフォーマット仕様**
   - `active.yaml` のスキーマと用途
   - `tasks.yaml` のスキーマと用途
   - `status.yaml` のスキーマと用途
   - `report.md` のフォーマット

3. **ワークフロー**
   - 設計フェーズ（`/orchestrate:design`）
   - 実行フェーズ（`/orchestrate:start`）
   - 監視フェーズ
   - 完了フェーズ（`/orchestrate:cleanup`）

4. **コマンド一覧**
   - 各コマンドの概要と使用タイミング

5. **ワーカーの制約**
   - 編集可能なファイル
   - 禁止事項

6. **トラブルシューティング**
   - よくある問題と対処法

### ファイルフォーマット詳細

#### active.yaml

```yaml
# 現在進行中の並列オーケストレーション
active_feature:
  feature_id: string      # Feature ID
  started_at: datetime    # 開始日時（ISO 8601）
  orchestrator_pid: int   # オーケストレーターのプロセスID
```

#### tasks.yaml

```yaml
feature_id: string        # Feature ID
base_branch: string       # ベースブランチ名

tasks:
  - id: string            # タスクID（ディレクトリ名と一致）
    name: string          # タスク名（表示用）
    dependencies: [string] # 依存タスクIDのリスト
```

#### status.yaml

```yaml
status: string            # pending | in_progress | completed | failed
feature_branch: string    # Feature ブランチ名
started_at: datetime      # 開始日時
updated_at: datetime      # 最終更新日時

active_tasks:             # 現在実行中のタスク
  - task_id: string
    worktree_path: string
    branch: string
    worker_pid: int
    started_at: datetime

completed_tasks: [string] # 完了済みタスクID
pending_tasks: [string]   # 待機中タスクID
```

---

## 完了条件

- [ ] `.claude/skills/parallel-orchestration/SKILL.md` が作成されている
- [ ] 機構の概要が説明されている
- [ ] 全ファイルフォーマットのスキーマが定義されている
- [ ] ワークフローが図示されている
- [ ] ワーカーの制約が明記されている

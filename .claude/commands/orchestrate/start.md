---
description: オーケストレーター起動
argument-hint: [feature-id]
---

# 並列オーケストレーション起動コマンド

**対象 Feature:** $ARGUMENTS

---

## このコマンドの目的

設計済みの Feature に対して、並列タスク実行を開始する。
オーケストレーターとして、依存関係を解決しながらワーカーをディスパッチし、全タスクの完了まで監視する。

**前提:**
- `/orchestrate:design` により `tasks.yaml`, 各タスクの `spec.md` が生成済みであること
- Feature ブランチが作成済み、または作成可能であること

---

## 事前参照ドキュメント

以下のドキュメントを参照して、機構の全体像を理解すること:

1. **AGENTS.md** - プロジェクト概要
2. **SKILL.md** - `.claude/skills/parallel-orchestration/SKILL.md`
3. **オーケストレーターの振る舞い** - `.claude/skills/parallel-orchestration/references/orchestrator-behavior.md`
4. **ワーカー用プロンプトテンプレート** - `.claude/skills/parallel-orchestration/appendix/worker-prompt.md`

---

## 実行フェーズ

### Phase 1: 初期化

#### 1.1 排他制御の確認

`.agents/active.yaml` を確認し、他の Feature が進行中でないことを確認する。

```yaml
# 進行中がない場合
active_feature: null

# 進行中がある場合 → 警告して中断
active_feature:
  feature_id: {other-feature-id}
  started_at: ...
```

**進行中の場合:**
```
⚠️ 他の Feature が進行中です: {feature-id}

現在の状態を確認するには `.agents/active.yaml` を確認してください。
進行中の Feature を完了させるか、手動でクリアしてから再実行してください。
```

#### 1.2 タスク定義の読み込み

`tasks.yaml` を読み込み、依存グラフを構築する。

```
.agents/features/{feature-id}/tasks.yaml
```

**tasks.yaml が見つからない場合:**
```
❌ tasks.yaml が見つかりません

`/orchestrate:design` を実行して、タスク定義を作成してください。
```

**循環依存がある場合:**
```
❌ 循環依存が検出されました

以下のタスク間で循環が発生しています:
- {task-a} → {task-b} → {task-c} → {task-a}

tasks.yaml を修正して、依存関係を整理してください。
```

#### 1.3 Feature ブランチの確認・作成

現在のブランチを確認し、必要に応じて Feature ブランチを作成または切り替える。

```bash
# 現在のブランチを確認
git branch --show-current
```

- 現在のブランチが `feature/{feature-id}` → そのまま継続
- それ以外 → Feature ブランチを作成または切り替え

```bash
# ブランチが存在しない場合
git checkout -b feature/{feature-id}

# ブランチが存在する場合
git checkout feature/{feature-id}
```

**補足:** 通常 `/orchestrate:design` 実行時に Feature ブランチは作成済み。

#### 1.4 状態ファイルの初期化

**active.yaml を更新:**

```yaml
active_feature:
  feature_id: {feature-id}
  started_at: {ISO 8601 datetime}
  orchestrator_pid: {current pid}
```

**status.yaml を作成:**

```yaml
status: in_progress
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

---

### Phase 2: ディスパッチ

#### 2.1 依存解消タスクの抽出

`tasks.yaml` を参照し、以下の条件を満たすタスクを抽出:

- `dependencies` が空配列、または
- `dependencies` の全タスクが `completed_tasks` に含まれる

#### 2.2 各タスクに対してワーカーを起動

[`start-worker.sh`](../../skills/parallel-orchestration/scripts/start-worker.sh) を使用して worktree 作成・ワーカー起動を一括で行う。

```bash
WORKER_PID=$(bash .claude/skills/parallel-orchestration/scripts/start-worker.sh {task-id})
```

---

### Phase 3: 監視ループ

#### 3.1 監視スクリプトの起動

[`wait-for-completion.sh`](../../skills/parallel-orchestration/scripts/wait-for-completion.sh) を Bash ツールの `run_in_background=true` で起動する。

```bash
bash .claude/skills/parallel-orchestration/scripts/wait-for-completion.sh
```

スクリプトが exit すると、オーケストレーターに自動通知が届く。

#### 3.2 タスク完了検知時の処理

**a. タスクブランチを Feature ブランチにマージ:**

オーケストレーターがエージェンティックにマージを行う。
コンフリクトが発生した場合は内容を判断して解決する。

```bash
git checkout feature/{feature-id}
git merge task/{feature-id}_{task-id}
```

**b. クリーンアップ:**

マージ完了後、[`complete-task.sh`](../../skills/parallel-orchestration/scripts/complete-task.sh) でクリーンアップを行う。

```bash
bash .claude/skills/parallel-orchestration/scripts/complete-task.sh {task-id}
```

**c. 依存解消チェック:**

`tasks.yaml` を参照し、新たに unblocked になったタスクがあれば Phase 2 へ戻りディスパッチする。

#### 3.3 ループ継続判定

- `active_tasks` が空でなければ `wait-for-completion.sh` を再起動してループを継続
- 全タスクが `completed_tasks` に含まれたら Phase 4 へ

---

### Phase 4: 完了処理

#### 4.1 status.yaml を更新

```yaml
status: completed
updated_at: {ISO 8601 datetime}

active_tasks: []

completed_tasks:
  - {全タスクID}

pending_tasks: []
```

#### 4.2 active.yaml をクリア

```yaml
active_feature: null
```

#### 4.3 完了レポートを出力

```
✅ Feature の並列実行が完了しました

Feature: {feature-id}
完了タスク数: {n}

次のステップ:
1. 各タスクの report.md を確認
2. Feature ブランチで統合テストを実行
3. PR を作成: gh pr create --base main --head feature/{feature-id}
```

---

## エラーハンドリング

### 起動時のエラー

| エラー | 対処 |
|--------|------|
| 既に他 Feature が進行中 | 警告して中断、`active.yaml` の確認を案内 |
| `tasks.yaml` が見つからない | エラー、`/orchestrate:design` を案内 |
| 循環依存の検出 | エラー、`tasks.yaml` の修正を案内 |

### 実行時のエラー

| エラー | 対処 |
|--------|------|
| ワーカーがクラッシュ | `status.yaml` に失敗を記録、ユーザーに報告 |
| worktree 作成失敗 | エラーログを出力、手動対応を案内 |
| マージコンフリクト | 中断、手動解決を案内 |

---

## 命名規則

| 対象 | 形式 | 例 |
|------|------|-----|
| Feature ブランチ | `feature/{feature-id}` | `feature/20260209-parallel_orchestration` |
| タスクブランチ | `task/{feature-id}_{task-id}` | `task/20260209-parallel_orchestration_01-skill-definition` |
| worktree | `.worktrees/task_{feature-id}_{task-id}/` | `.worktrees/task_20260209-parallel_orchestration_01-skill-definition/` |

---

## 制約事項

### 編集可能

| 対象 | 説明 |
|------|------|
| `active.yaml` | 進行中 Feature の管理 |
| `status.yaml` | 実行状態の管理 |
| worktree の作成・削除 | `background-developing-with-worktree` skill に委譲 |
| ブランチの作成・マージ・削除 | Feature/タスクブランチの管理 |

### 編集禁止

| 対象 | 理由 |
|------|------|
| `tasks.yaml` | 静的定義（設計時に確定） |
| `spec.md`（Feature, タスク） | 仕様の変更禁止 |
| `report.md` | ワーカー専用 |
| ソースコード | ワーカー専用 |

### 禁止操作

| 操作 | 理由 |
|------|------|
| タスクの実装を行う | ワーカーの責務 |
| `report.md` を作成・編集する | ワーカー専用 |
| 依存関係を無視したディスパッチ | タスク間の整合性を破壊 |

---

## 参照ドキュメント

- [並列オーケストレーション機構](../../skills/parallel-orchestration/SKILL.md) - 機構の全体像
- [オーケストレーターの振る舞い](../../skills/parallel-orchestration/references/orchestrator-behavior.md) - 詳細な動作仕様
- [ワーカー用プロンプトテンプレート](../../skills/parallel-orchestration/appendix/worker-prompt.md) - ワーカー起動時のプロンプト
- [status.yaml フォーマット](../../skills/parallel-orchestration/appendix/status-yaml.md) - status.yaml のスキーマ
- [active.yaml フォーマット](../../skills/parallel-orchestration/appendix/active-yaml.md) - active.yaml のスキーマ

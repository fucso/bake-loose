# tasks.yaml テンプレート

タスク定義と依存関係を記述する静的ファイル。`/orchestrate:design` で生成される。

## 配置場所

```
.agents/features/{feature-id}/tasks.yaml
```

---

## 基本構造

```yaml
feature_id: {feature-id}
base_branch: main

tasks:
  - id: {task-id}
    name: {タスク名}
    dependencies: []

  - id: {task-id}
    name: {タスク名}
    dependencies:
      - {依存タスクID}
```

## 例

```yaml
feature_id: 20260209-parallel_orchestration
base_branch: main

tasks:
  - id: 01-skill-definition
    name: スキル定義
    dependencies: []

  - id: 02-design-command
    name: 設計コマンド
    dependencies:
      - 01-skill-definition

  - id: 03-worker-agent
    name: ワーカーエージェント
    dependencies:
      - 01-skill-definition

  - id: 04-orchestrator-start
    name: オーケストレーター起動
    dependencies:
      - 01-skill-definition
      - 03-worker-agent

  - id: 05-integration
    name: 統合テスト
    dependencies:
      - 02-design-command
      - 03-worker-agent
      - 04-orchestrator-start
```

## フィールド説明

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `feature_id` | string | Feature ID（ディレクトリ名と一致） |
| `base_branch` | string | ベースブランチ名（通常は main） |
| `tasks` | array | タスクの配列 |
| `tasks[].id` | string | タスクID（`{番号}-{名前}` 形式、ディレクトリ名と一致） |
| `tasks[].name` | string | タスク名（表示用） |
| `tasks[].dependencies` | array | 依存タスクIDのリスト（空配列可） |

## 命名規則

### タスクID

```
{2桁番号}-{kebab-case名}
```

例:
- `01-skill-definition`
- `02-design-command`
- `03-worker-agent`

### 依存関係の記述

- 依存がない場合: 空配列 `[]`
- 依存がある場合: タスクIDの配列

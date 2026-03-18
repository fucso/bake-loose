# active.yaml テンプレート

現在進行中の Feature を管理するグローバルファイル。

## 配置場所

```
.agents/active.yaml
```

## フィールド説明

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `active_feature` | object/null | 進行中の Feature 情報（未実行時は null） |
| `active_feature.feature_id` | string | Feature ID |
| `active_feature.started_at` | datetime | 開始日時（ISO 8601） |
| `active_feature.orchestrator_pid` | int | オーケストレーターのプロセスID |

---

## 初期状態（未実行時）

```yaml
active_feature: null
```

## 実行中

```yaml
active_feature:
  feature_id: {feature-id}
  started_at: {ISO 8601 datetime}
  orchestrator_pid: {pid}
```

## 例

```yaml
active_feature:
  feature_id: 20260209-parallel_orchestration
  started_at: 2026-02-09T10:30:00+09:00
  orchestrator_pid: 12345
```

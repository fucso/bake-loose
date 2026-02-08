# Task: 設計コマンド

> Feature: [parallel_orchestration](../../spec.md)
> 依存: 01-skill-definition

## 目的

並列オーケストレーション用の Feature 設計コマンド `/orchestrate:design` を作成する。
既存の `/design` コマンドを拡張し、`tasks.yaml` も同時に生成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.claude/commands/orchestrate/design.md` | 新規 | 設計コマンド定義 |

---

## 設計詳細

### コマンド概要

```
/orchestrate:design [実現したい機能・要件]
```

### 出力ファイル

```
.agents/features/{yyyymmdd}-{feature_name}/
├── spec.md                        # Feature 仕様
├── tasks.yaml                     # タスク定義（新規）
└── tasks/
    ├── 01-{task-name}/
    │   └── spec.md
    └── ...
```

### 処理フロー

1. **Phase 1: 要件分析**（既存 `/design` と同様）
   - 機能要件・非機能要件の整理
   - 影響範囲の特定

2. **Phase 2: オープンクエスチョンの解決**
   - 不明点の解決

3. **Phase 3: タスク分解**
   - 依存関係を考慮したタスク分割
   - 並列実行可能なタスクの特定

4. **Phase 4: ファイル出力**
   - `spec.md` の生成（既存形式）
   - `tasks.yaml` の生成（新規）
   - 各タスクの `spec.md` 生成

### tasks.yaml 生成ルール

- `spec.md` のタスク一覧テーブルから自動生成
- タスクID: ディレクトリ名（`01-xxx` 形式）
- タスク名: テーブルの「タスク」列
- 依存関係: テーブルの「依存」列をパース
  - `-` → 空配列
  - `01` → `["01-xxx"]`（ID 形式に変換）
  - `01, 02` → `["01-xxx", "02-xxx"]`

### 既存 /design との違い

| 項目 | /design | /orchestrate:design |
|------|---------|---------------------|
| 出力 | `spec.md` + タスク `spec.md` | 同左 + `tasks.yaml` |
| タスク分解 | 依存関係を記載 | 依存関係を構造化データで出力 |
| 用途 | 逐次実行 | 並列実行 |

---

## 参照すべきドキュメント

- `.claude/commands/design.md`（既存設計コマンド）
- `.claude/skills/parallel-orchestration/SKILL.md`（ファイルフォーマット）
- `.claude/skills/spec-writing/SKILL.md`（仕様記載ルール）

---

## 完了条件

- [ ] `.claude/commands/orchestrate/design.md` が作成されている
- [ ] 処理フローが定義されている
- [ ] `tasks.yaml` の生成ルールが明記されている
- [ ] 既存 `/design` との差分が明確

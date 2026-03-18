---
description: 並列実行用の設計（spec.md + tasks.yaml を同時生成）
argument-hint: [実現したい機能・要件]
---

# 並列オーケストレーション設計コマンド

**ユーザーの要件:** $ARGUMENTS

---

## このコマンドの目的

ユーザーの機能要件を分析し、並列実行可能なタスク群として設計する。
`tasks.yaml` を生成することで、オーケストレーターによる並列実行を可能にする。

**出力:**
- `spec.md`: 要件分析とタスク分解
- `tasks.yaml`: 依存関係を構造化したタスク定義
- 各タスクの `spec.md`

---

## 出力ファイル構成

```
.agents/features/{yyyymmdd}-{feature_name}/
├── spec.md                        # Feature 仕様（要件分析 + タスク分解）
├── tasks.yaml                     # タスク定義・依存関係
└── tasks/
    ├── 01-{task-name}/
    │   └── spec.md                # タスク仕様
    ├── 02-{task-name}/
    │   └── spec.md
    └── ...
```

---

## 事前準備

以下のドキュメントを参照して、プロジェクトの設計思想を理解すること:

1. **AGENTS.md** - プロジェクト概要とアーキテクチャ
2. **各レイヤーのコーディングルール** - `.claude/rules/` 配下
3. **並列オーケストレーション機構** - `.claude/skills/parallel-orchestration/SKILL.md`
4. **目標定義ガイドライン** - `.claude/skills/defining-feature-task/references/feature-goal.md`, `.claude/skills/defining-feature-task/references/task-goal.md`

---

## Phase 1: 要件分析

### 1.1 要件の整理

ユーザーの要件を以下の観点で整理する:

- **機能要件**: 何ができるようになるか
- **非機能要件**: パフォーマンス、セキュリティ等（該当する場合）

### 1.2 影響範囲の特定

各レイヤー/コンポーネントへの影響を判断する:

| 対象 | 確認ポイント |
|------|--------------|
| domain | 新規モデル or 既存モデル変更 or 新規アクションが必要か |
| ports | 新規リポジトリメソッドが必要か |
| use_case | 新規ユースケースが必要か |
| repository | DB操作の実装が必要か |
| presentation | GraphQL スキーマ/リゾルバーの変更が必要か |
| migration | テーブル追加/変更が必要か |
| commands | 新規コマンドが必要か |
| skills | 新規スキルが必要か |
| agents | 新規エージェント定義が必要か |

---

## Phase 2: オープンクエスチョンの解決

要件に不明点がある場合は、**実装前に解決する**。

### 対話で解決すべき問題

- 仕様の曖昧さ（例: 上限値、許可される操作）
- 複数の実装方針がある場合の選択
- 既存機能との整合性

### 先送り可能な問題

実装を進めないと判断が難しい問題は、ユーザーと合意の上で先送りできる。
その場合、spec.md のオープンクエスチョンに記録する。

**質問例:**
```
以下について確認させてください:

1. {具体的な質問}
2. {具体的な質問}

また、以下は実装を進めながら判断する方針でよいでしょうか:
- {先送り候補の問題}
```

---

## Phase 3: Feature 目標の定義

[Feature 目標の定義](../../skills/defining-feature-task/references/feature-goal.md) に従い、Feature の完了目標を定義する。

### 定義手順

1. Feature の種別を判断する（バックエンド / フロントエンド / リファクタリング）
2. 振る舞いレベルの目標を正常系・異常系で記述する
3. 検証方法を明記する（どのテストがパスすれば達成か）

### チェックポイント

- [ ] 目標が実装の手段ではなく振る舞いで書かれている
- [ ] 正常系と異常系の両方が定義されている
- [ ] テストやコマンドで検証可能な形になっている

---

## Phase 4: タスク分解

### 分解の判断基準

**分けるケース:**
- レイヤーをまたぐ大きな変更 → レイヤー別に分割
- 複数エンティティに影響 → エンティティ別に分割
- migration が必要 → 独立タスクに
- 並列実行可能なタスク → 別タスクに分割

**分けないケース:**
- 1-2ファイルの小規模な修正 → 1タスクにまとめる
- 分けると依存が複雑になる場合 → まとめる

### タスク目標の定義

各タスクに対し、[Task 目標の定義](../../skills/defining-feature-task/references/task-goal.md) に従って完了目標を定義する。

1. **実装目標**: そのタスクの実装が正しく機能していることを具体的に記述
2. **テスト目標**: 追加・修正するテストとその期待結果を明記

**整合性チェック**: 全タスクの目標が達成されたとき、Feature の目標が達成されることを確認する。

### 並列実行を意識したタスク設計

- **依存関係の最小化**: タスク間の依存を減らし、並列実行可能なタスクを増やす
- **独立性の確保**: 各タスクが他タスクの完了を待たずに開始できるよう設計
- **共通基盤の先行**: 複数タスクが依存する共通部分は最初に実装

### タスクの命名規則

`{nn}-{layer or component}-{action}`

例:
- `01-skill-definition` - スキル定義
- `02-domain-model` - モデル定義
- `03-domain-action` - アクション実装
- `04-ports` - リポジトリトレイト
- `05-use-case` - ユースケース
- `06-repository` - リポジトリ実装
- `07-presentation` - GraphQL
- `08-integration` - 統合テスト

---

## Phase 5: ファイル出力

### feature ディレクトリの命名

`{yyyymmdd}-{feature_name}` の形式で命名する:
- 日付部分: 設計実行日（例: `20260209`）
- feature_name: スネークケースで要件を端的に表す

例:
- `20260209-add_trial_photo`
- `20260209-parallel_orchestration`

### 4.1 spec.md の生成

[仕様記載ルール](../../skills/defining-feature-task/SKILL.md) に従い、以下を含める:

- 概要
- 元の要件
- 要件分析（機能要件・非機能要件）
- 影響範囲
- アーキテクチャ（必要に応じて）
- タスク分解（分解方針・タスク一覧・実装順序図）
- 前提条件
- オープンクエスチョン（あれば）

### 4.2 tasks.yaml の生成

spec.md のタスク一覧から `tasks.yaml` を生成する。
フォーマットと記載例は [tasks.yaml フォーマット](../../skills/parallel-orchestration/appendix/tasks-yaml.md) を参照。

### 4.3 各タスクの spec.md 生成

各タスクディレクトリに spec.md を作成する。

**必須項目:**

- タスク名とリンク（Feature spec.md へ）
- 目的
- 変更対象（ファイル一覧）
- 設計詳細
- 完了条件（チェックリスト形式）

---

## テストの配置ルール

| レイヤー | テスト配置 | 備考 |
|----------|-----------|------|
| domain | `src/domain/actions/{entity}/{action}.rs` 内 | `#[cfg(test)] mod tests` |
| ports | なし | トレイト定義のみのためテスト不要 |
| use_case | `src/use_case/{entity}/{use_case}.rs` 内 | `MockUnitOfWork` 使用 |
| repository | `src/repository/{entity}_repo.rs` 内 | `#[sqlx::test]` 使用 |
| presentation | `tests/graphql/{entity}/{operation}.rs` | 統合テスト |

---

## 注意事項

- **実装は行わない**: このコマンドは設計のみ
- **オープンクエスチョンは解決してから進む**: 先送りする場合は合意を得て記録する
- **スキルドキュメントを参照**: 各レイヤーのルールに従った設計を行う
- **並列実行を意識**: 依存関係を最小化し、可能な限り並列実行できるようタスクを設計する
- **tasks.yaml の整合性**: spec.md のタスク一覧と tasks.yaml が一致していることを確認する

---

## 参照ドキュメント

- [仕様記載ルール](../../skills/defining-feature-task/SKILL.md) - spec.md の記載ガイドライン
- [Feature 目標の定義](../../skills/defining-feature-task/references/feature-goal.md) - Feature の振る舞いレベル目標
- [Task 目標の定義](../../skills/defining-feature-task/references/task-goal.md) - Task の実装レベル目標
- [並列オーケストレーション機構](../../skills/parallel-orchestration/SKILL.md) - 機構の全体像
- [tasks.yaml フォーマット](../../skills/parallel-orchestration/appendix/tasks-yaml.md) - tasks.yaml のスキーマ

---
description: Claude Code on the Web で実装タスクを実行する
argument-hint: <source-branch> <task-spec-path>
---

# Dev on Web コマンド

**引数:** $ARGUMENTS

---

## このコマンドの目的

Claude Code on the Web 上でのタスクを実装を行う。
main ブランチから開始し、指定されたブランチにチェックアウトしてタスクを実装、PR を作成する。

**重要**: Web 環境ではユーザーとの対話ができないため、方針確認やレビュー承認なしで自動実行する。

---

## 引数の解析

引数は以下の形式で渡される:
- 第1引数: フォーク元ブランチ（例: `feature/20260110-create_trial`）
- 第2引数: タスク仕様パス（例: `.agents/features/20260110-create_trial/tasks/01-migration`）

引数をパースして以下の変数として扱う:
- `SOURCE_BRANCH`: 第1引数
- `TASK_SPEC_PATH`: 第2引数
- `TASK_BRANCH`: `task/{feature-name}-{task-name}` 形式で生成（例: `task/20260110-create_trial-01-migration`）

---

## 前提条件

- フォーク元ブランチが存在すること
- タスク仕様ファイルが存在すること
- 依存タスクがすべて完了していること
- `GITHUB_TOKEN` 環境変数が設定されていること

---

## 事前準備

### 1. ブランチの準備

フォーク元ブランチからタスク用ブランチを作成する。

```bash
git fetch origin
git checkout $SOURCE_BRANCH
git pull origin $SOURCE_BRANCH
git checkout -b $TASK_BRANCH
```

例:
- `SOURCE_BRANCH`: `feature/20260110-create_trial`
- `TASK_BRANCH`: `task/20260110-create_trial-01-migration`

### 2. AGENTS.md の読み込み

プロジェクト構造と設計思想を把握するため AGENTS.md を読む。

### 3. タスク仕様の読み込み

指定されたパスの `spec.md` を読み込む。

### 4. Feature 仕様の確認

タスク仕様から Feature の `spec.md` を参照し、全体の文脈を把握する。

### 5. 依存タスクの確認

タスク仕様に記載された依存タスクがあれば、その `report.md` で実装状況を確認する。
依存タスクが未完了の場合は、エラーメッセージを出力して中断する。

---

## 実装フロー

### Phase 1: 実装

タスク仕様とルールに従って実装を行う。

**実装時の注意事項:**

1. **スキルドキュメントのルールに従う**
   - 各レイヤーの責務を守る
   - アンチパターンを避ける
   - ファイル配置規約に従う

2. **タスク仕様の完了条件を満たす**
   - spec.md の「完了条件」を一つずつ確認
   - すべての条件を満たすまで実装

3. **モジュール宣言の追加**
   - 新規ファイル作成時は親モジュールファイルに `pub mod xxx;` 宣言を追加
   - 例: `src/domain/models/project.rs` を作成 → `src/domain/models.rs` に `pub mod project;` を追加
   - ※ `mod.rs` は使用しない（Rust 2018 edition 以降の推奨に従う）

### Phase 2: 品質チェック

**重要**: backend ディレクトリで直接コマンドを実行する。

```bash
cd backend

# フォーマット（自動整形）
cargo fmt

# Lint（警告をエラー扱い）
cargo clippy --all-targets -- -D warnings

# テスト
cargo test
```

**エラー時の対応:**
- エラーがあれば修正を試みる
- 3回試行しても解決しない場合は、エラー内容を PR の body に記載して続行する

### Phase 3: Git コミット

変更内容を Git コミットする。

**コミット手順**:

1. `git status` で変更ファイルを確認
2. `git add` で変更をステージング
3. 以下の形式でコミットメッセージを作成してコミット

**コミットメッセージ形式**:

```
feat({feature-id}): {タスク名}

- {変更内容1}
- {変更内容2}
- ...

Task: {task-name}
```

**例**:

```
feat(create_trial): ドメインモデル定義

- Trial 構造体を追加
- TrialId 値オブジェクトを追加
- Trial::new() コンストラクタを実装

Task: 01-domain-model
```

### Phase 4: Push と PR 作成

`.claude/skills/web/create-pr` スキルのワークフローに従って push と PR 作成を行う。

- **ベースブランチ**: `$SOURCE_BRANCH`
- **ヘッドブランチ**: `$TASK_BRANCH`
- **PR タイトル**: `feat({feature-id}): {タスク名}`
- **PR 本文**: 以下の形式で作成

```markdown
## Summary

- {実装概要を箇条書き}

## Task Spec

`.agents/features/{feature-id}/tasks/{task-name}/spec.md`

## Changes

| ファイル | 操作 | 概要 |
|----------|------|------|
| ... | 新規/修正 | ... |

## Quality Checks

- [x] `cargo fmt` - フォーマット済み
- [x] `cargo clippy` - 警告なし
- [x] `cargo test` - 全テスト通過

## Test Plan

- [ ] CI パイプラインが成功することを確認

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## エラーハンドリング

### タスク仕様が見つからない場合

エラーメッセージを出力して中断:

```
ERROR: 指定されたパスにタスク仕様が見つかりませんでした: {path}
```

### 依存タスクが未完了の場合

エラーメッセージを出力して中断:

```
ERROR: 依存タスクが未完了です: {依存タスク名}
```

### コンパイルエラー/テスト失敗の場合

1. エラー内容を分析し、修正を試みる
2. 3回試行しても解決しない場合:
   - PR の body に未解決のエラー内容を記載
   - PR タイトルに `[WIP]` プレフィックスを付与
   - そのまま PR を作成して続行

---

## 注意事項

- **タスク仕様に忠実に**: 仕様にない変更は行わない
- **スコープを守る**: このタスクの範囲外の変更は次のタスクで行う
- **対話なし**: ユーザー確認を求めず自動実行する

# Task: クリーンアップコマンド

> Feature: [parallel_orchestration](../../spec.md)
> 依存: 01-skill-definition

## 目的

並列オーケストレーション完了後のクリーンアップを行うコマンド `/orchestrate:cleanup` を作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.claude/commands/orchestrate/cleanup.md` | 新規 | クリーンアップコマンド |

---

## 設計詳細

### コマンド概要

```
/orchestrate:cleanup {feature-id}
```

### 処理フロー

#### Phase 1: 状態確認

1. `status.yaml` を読み込む
2. ステータスが `completed` または `failed` であることを確認
   - `in_progress` の場合は警告して中断

#### Phase 2: worktree 削除

1. `.agents/worktrees/` 配下の該当 Feature の worktree を削除
   ```bash
   git worktree remove .agents/worktrees/{feature-id}_{task-id} --force
   ```
2. 削除した worktree の一覧を出力

#### Phase 3: ブランチ整理

1. タスクブランチを Feature ブランチにマージ（オプション）
   - ユーザーに確認を取る
   - マージする場合:
     ```bash
     git checkout feature/{feature-id}
     git merge task/{feature-id}_{task-id}
     ```
2. タスクブランチを削除
   ```bash
   git branch -d task/{feature-id}_{task-id}
   ```

#### Phase 4: 状態ファイル更新

1. `.agents/active.yaml` をクリア（まだの場合）
2. `status.yaml` はそのまま保持（履歴として）

#### Phase 5: 完了レポート

```
## クリーンアップ完了

**Feature:** {feature-id}

### 削除した worktree

- {worktree_path_1}
- {worktree_path_2}
- ...

### 削除したブランチ

- {branch_1}
- {branch_2}
- ...

### Feature ブランチ

`feature/{feature-id}` に全タスクの変更がマージされています。

次のステップ:
- `git checkout feature/{feature-id}` でブランチを確認
- PR を作成して main にマージ
```

### オプション

| オプション | 説明 |
|------------|------|
| `--force` | `in_progress` 状態でも強制的にクリーンアップ |
| `--no-merge` | タスクブランチのマージをスキップ |
| `--keep-branches` | タスクブランチを削除しない |

### エラーハンドリング

| エラー | 対処 |
|--------|------|
| Feature が見つからない | エラー、利用可能な Feature 一覧を表示 |
| まだ進行中 | 警告、`--force` オプションを案内 |
| worktree 削除失敗 | 警告を出力して続行、手動削除を案内 |
| マージコンフリクト | 中断、手動解決を案内 |

---

## 完了条件

- [ ] `.claude/commands/orchestrate/cleanup.md` が作成されている
- [ ] worktree 削除の手順が定義されている
- [ ] ブランチ整理の手順が定義されている
- [ ] オプションが定義されている
- [ ] エラーハンドリングが定義されている

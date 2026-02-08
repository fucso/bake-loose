# Task: 統合テスト

> Feature: [parallel_orchestration](../../spec.md)
> 依存: 02-design-command, 03-worker-agent, 04-orchestrator-start, 05-orchestrator-status, 06-orchestrator-cleanup

## 目的

並列オーケストレーション機構全体の統合テストを実施し、動作を検証する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `.agents/features/test-parallel/` | 新規 | テスト用 Feature |
| ドキュメント更新 | 修正 | 検証結果に基づく修正 |

---

## 設計詳細

### テストシナリオ

#### シナリオ 1: 基本的なワークフロー

1. `/orchestrate:design` でテスト用 Feature を作成
   - 2-3 個の簡単なタスク（ファイル作成程度）
   - 依存関係あり（直列）

2. `/orchestrate:start` で実行開始
   - worktree が正しく作成されるか
   - ワーカーが起動するか
   - `active.yaml`, `status.yaml` が正しく更新されるか

3. `/orchestrate:status` で進捗確認
   - 正しいステータスが表示されるか

4. 全タスク完了を待つ
   - `report.md` が作成されるか
   - 依存解決が正しく行われるか

5. `/orchestrate:cleanup` でクリーンアップ
   - worktree が削除されるか
   - ブランチが整理されるか

#### シナリオ 2: 並列実行

1. 依存関係のない 2 つのタスクを含む Feature を作成
2. 両タスクが同時にディスパッチされるか確認
3. 両タスク完了後、依存する第 3 タスクがディスパッチされるか確認

#### シナリオ 3: エラーハンドリング

1. タスク実行中に `/orchestrate:status` を実行
2. 進行中に `/orchestrate:cleanup` を実行（警告確認）
3. 存在しない Feature ID を指定（エラー確認）

### テスト用 Feature 構成

```
.agents/features/test-parallel/
├── spec.md
├── tasks.yaml
└── tasks/
    ├── 01-create-file-a/
    │   └── spec.md
    ├── 02-create-file-b/
    │   └── spec.md
    └── 03-merge-files/
        └── spec.md
```

**tasks.yaml:**
```yaml
feature_id: "test-parallel"
base_branch: "main"

tasks:
  - id: "01-create-file-a"
    name: "ファイルA作成"
    dependencies: []

  - id: "02-create-file-b"
    name: "ファイルB作成"
    dependencies: []

  - id: "03-merge-files"
    name: "ファイル統合"
    dependencies:
      - "01-create-file-a"
      - "02-create-file-b"
```

### 検証項目チェックリスト

#### ファイル操作

- [ ] `active.yaml` が正しく更新される
- [ ] `status.yaml` が正しく更新される
- [ ] worktree が正しく作成・削除される
- [ ] `report.md` が正しく作成される

#### 依存解決

- [ ] 依存のないタスクが同時にディスパッチされる
- [ ] 依存タスク完了後に後続タスクがディスパッチされる
- [ ] 循環依存が検出される（該当ケースがあれば）

#### エラーハンドリング

- [ ] 不正な入力に対して適切なエラーメッセージが表示される
- [ ] 進行中の cleanup 試行に対して警告される

#### ログ・レポート

- [ ] `/orchestrate:status` が正しい情報を表示する
- [ ] クリーンアップ完了レポートが正しい

---

## 完了条件

- [ ] テスト用 Feature が作成されている
- [ ] 全シナリオが実行されている
- [ ] 検証項目チェックリストが全て確認されている
- [ ] 発見された問題が修正されている
- [ ] テスト結果が report.md に記録されている

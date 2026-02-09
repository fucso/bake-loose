# トラブルシューティング

並列オーケストレーション機構でよくある問題と対処法。

---

## 起動時の問題

### 「既に他の Feature が進行中」エラー

**症状:**
```
エラー: 既に 20260208-xxx が進行中です。
```

**原因:**
- `.agents/active.yaml` に別の Feature が登録されている

**対処:**
1. `.agents/active.yaml` と `status.yaml` で状況を確認
2. 進行中の Feature が完了していれば `active.yaml` を手動でクリア
3. 強制的に開始する場合は「手動リカバリー」セクションを参照

---

### 「tasks.yaml が見つからない」エラー

**症状:**
```
エラー: tasks.yaml が見つかりません。
```

**原因:**
- `/orchestrate:design` が実行されていない
- Feature ID が間違っている

**対処:**
1. Feature ID を確認
2. `/orchestrate:design` で設計を実行

---

### 「循環依存が検出されました」エラー

**症状:**
```
エラー: 循環依存が検出されました: 01 → 02 → 03 → 01
```

**原因:**
- `tasks.yaml` の `dependencies` が循環している

**対処:**
1. `tasks.yaml` を確認
2. 循環を解消するように依存関係を修正

---

## 実行中の問題

### ワーカーが応答しない

**症状:**
- タスクが長時間 `active_tasks` に残っている
- `report.md` が作成されない

**原因:**
- ワーカープロセスがハングしている
- 無限ループやデッドロック

**対処:**
1. ワーカーのログを確認（`{task_dir}/worker_output.log`）
2. プロセスを手動で終了
   ```bash
   kill {worker_pid}
   ```
3. 「手動リカバリー」セクションを参照してリセット

---

### ビルドエラーでワーカーが失敗

**症状:**
- `report.md` にビルドエラーが記載されている
- `status.yaml` が `failed` になっている

**対処:**
1. `report.md` でエラー内容を確認
2. 手動でコードを修正
3. 該当タスクを手動で再実行
4. 完了後に `report.md` を更新

---

### テストエラーでワーカーが失敗

**症状:**
- `report.md` にテストエラーが記載されている

**対処:**
1. `report.md` で失敗したテストを確認
2. 手動でコードまたはテストを修正
3. Docker Compose 環境でテストを再実行
   ```bash
   docker compose exec backend bash -c "cargo test"
   ```
4. `report.md` を更新

---

## worktree の問題

### worktree の削除に失敗

**症状:**
```
エラー: worktree '{path}' を削除できません
```

**原因:**
- worktree 内でプロセスが実行中
- ファイルがロックされている

**対処:**
1. worktree 内のプロセスを終了
2. 手動で削除
   ```bash
   git worktree remove {path} --force
   ```
3. それでも失敗する場合はディレクトリを直接削除
   ```bash
   rm -rf {path}
   git worktree prune
   ```

---

### worktree 内の変更が消えた

**症状:**
- worktree で作業した内容が見つからない

**原因:**
- コミットせずに worktree を削除した
- 別のブランチにコミットした

**対処:**
1. `git reflog` で履歴を確認
2. `git cherry-pick` で変更を復元

---

## ブランチの問題

### マージコンフリクト

**症状:**
```
エラー: マージコンフリクトが発生しました
```

**原因:**
- 複数のタスクが同じファイルを変更した

**対処:**
1. Feature ブランチをチェックアウト
   ```bash
   git checkout feature/{feature-id}
   ```
2. コンフリクトを手動で解決
3. コミット
   ```bash
   git add .
   git commit -m "Resolve merge conflict"
   ```

---

### タスクブランチが見つからない

**症状:**
```
エラー: ブランチ 'task/{feature-id}_{task-id}' が見つかりません
```

**原因:**
- ブランチが既に削除されている
- ブランチ名が間違っている

**対処:**
1. ブランチ一覧を確認
   ```bash
   git branch -a
   ```
2. 正しいブランチ名を使用

---

## 状態ファイルの問題

### status.yaml が壊れている

**症状:**
- YAML パースエラー
- オーケストレーターが status.yaml の読み込みに失敗

**対処:**
1. `status.yaml` をバックアップ
2. [appendix/status-yaml.md](../appendix/status-yaml.md) のテンプレートを参考に修正
3. または「手動リカバリー」セクションを参照してリセット

---

### active.yaml と status.yaml の不整合

**症状:**
- `active.yaml` は進行中を示すが `status.yaml` は completed

**対処:**
1. 実際の状態を確認
2. `active.yaml` を手動で修正
   ```yaml
   active_feature: null
   ```

---

## 手動リカバリー

全てが失敗した場合の手動リカバリー手順:

1. **全ワーカープロセスを終了**
   ```bash
   # status.yaml から PID を確認して終了
   kill {pid1} {pid2} ...
   ```

2. **worktree を全て削除**
   ```bash
   rm -rf .agents/worktrees/{feature-id}_*
   git worktree prune
   ```

3. **状態ファイルをリセット**
   ```bash
   # active.yaml をクリア
   echo "active_feature: null" > .agents/active.yaml

   # status.yaml を削除（履歴が不要な場合）
   rm .agents/features/{feature-id}/status.yaml
   ```

4. **ブランチを整理**
   ```bash
   git branch -D task/{feature-id}_*
   git branch -D feature/{feature-id}
   ```

5. **最初からやり直し**
   ```bash
   /orchestrate:start {feature-id}
   ```

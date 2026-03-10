# オーケストレーターの振る舞い

オーケストレーターの役割、実行手順、制約事項。

---

## 概要

オーケストレーターは Feature 全体のタスク実行を統制する。
依存関係を解決しながらワーカーをディスパッチし、ファイルベースで状態を管理する。

### 責務

- Feature の初期化（ブランチ確認・作成、状態ファイル作成）
- 依存グラフの構築と解決
- ワーカーのディスパッチ
- タスク完了の検知と後処理
- 状態管理（`active.yaml`, `status.yaml`）

---

## 実行手順

### Phase 1: 初期化

1. **排他制御の確認**
   - `.agents/active.yaml` を確認
   - 既に他の Feature が進行中なら警告して中断

2. **タスク定義の読み込み**
   - `tasks.yaml` を読み込み
   - 依存グラフを構築
   - 循環依存がないことを確認

3. **Feature ブランチの確認・作成**
   - 現在のブランチが `feature/{feature-id}` であればそのまま継続
   - そうでなければ Feature ブランチを作成
   ```bash
   # ブランチが存在しない場合のみ実行
   git checkout -b feature/{feature-id}
   ```

   **補足**: 通常、`/orchestrate:design` 実行時に Feature ブランチは作成済み。
   design 後にマージせずに start を実行した場合、ブランチ作成はスキップされる。

4. **状態ファイルの初期化**
   - `active.yaml` を更新（進行中を記録）
   - `status.yaml` を作成（初期状態）

### Phase 2: ディスパッチ

1. **依存解消タスクの抽出**
   - `dependencies` が空、または全て `completed_tasks` に含まれるタスクを抽出

2. **各タスクに対して:**
   a. **worktree を作成**
      ```bash
      git worktree add .agents/worktrees/{feature-id}_{task-id} -b task/{feature-id}_{task-id}
      ```

   b. **ワーカープロセスを起動**
      - [ワーカー用プロンプトテンプレート](../appendix/worker-prompt.md) を使用してプロンプトを構築

      **注意: Claude Code in Claude Code の実現**

      オーケストレーター自身も Claude Code セッションとして実行されるため、ワーカーを起動する際に `CLAUDECODE` 環境変数を継承させないよう `env -u CLAUDECODE` を使用する。

      ```bash
      cd {worktree_path} && env -u CLAUDECODE claude -p "{prompt}" > {task_dir}/worker_output.log 2>&1 &
      ```

      これにより、各ワーカーは独立した Claude Code セッションとして起動される。

   c. **status.yaml を更新**
      - `active_tasks` に追加
      - `pending_tasks` から削除

### Phase 3: 監視ループ

1. **監視スクリプトの起動**
   - `wait-for-completion.sh` を Bash ツールの `run_in_background=true` で起動
   ```bash
   bash .claude/skills/parallel-orchestration/scripts/wait-for-completion.sh
   ```
   - スクリプトは `active.yaml` と `status.yaml` から監視対象を自動取得
   - いずれかのタスクが完了（report.md コミット検知）またはクラッシュ（プロセス消失）した時点で exit
   - 出力: `COMPLETED:{task_id}` (exit 0) / `CRASHED:{task_id}` (exit 1)
   - ポーリング間隔: 30秒から徐々に増加（最大 300秒）

2. **タスク完了検知時:**
   a. **worktree を削除**
      ```bash
      git worktree remove .agents/worktrees/{feature-id}_{task-id} --force
      ```

   b. **タスクブランチを Feature ブランチにマージ**
      ```bash
      git checkout feature/{feature-id}
      git merge task/{feature-id}_{task-id}
      ```

   c. **タスクブランチを削除**
      ```bash
      git branch -d task/{feature-id}_{task-id}
      ```

   d. **status.yaml を更新**
      - `active_tasks` から削除
      - `completed_tasks` に追加

   e. **依存解消チェック**
      - 新たに unblocked になったタスクがあればディスパッチ（Phase 2 へ）

3. **ループ継続判定**
   - `active_tasks` が空でなければ `wait-for-completion.sh` を再起動してループを継続
   - 全タスク完了なら Phase 4 へ

### Phase 4: 完了処理

1. **status.yaml を更新**
   - `status: completed`

2. **active.yaml をクリア**
   - `active_feature: null`

3. **完了レポートを出力**
   - 次のステップを案内（PR 作成など）

---

## 状態管理

### 管理するファイル

| ファイル | 用途 | 更新タイミング |
|----------|------|---------------|
| `active.yaml` | 進行中 Feature の記録 | 開始時・完了時 |
| `status.yaml` | 実行状態の詳細 | 各フェーズの遷移時 |

---

## 命名規則

| 対象 | 形式 | 例 |
|------|------|-----|
| Feature ブランチ | `feature/{feature-id}` | `feature/20260209-parallel_orchestration` |
| タスクブランチ | `task/{feature-id}_{task-id}` | `task/20260209-parallel_orchestration_01-skill-definition` |
| worktree | `.agents/worktrees/{feature-id}_{task-id}/` | `.agents/worktrees/20260209-parallel_orchestration_01-skill-definition/` |

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

### リソース管理

| 課題 | 対処 |
|------|------|
| メモリ不足 | 並列実行数を減らす、一部タスクを直列化 |
| Docker コンテナのクラッシュ | `docker compose restart` でコンテナを再起動 |
| ディスク容量不足 | 古い worktree を削除、不要なファイルをクリーン |

---

## 制約事項

### 編集権限

#### 編集可能

| 対象 | 説明 |
|------|------|
| `active.yaml` | 進行中 Feature の管理 |
| `status.yaml` | 実行状態の管理 |
| worktree の作成・削除 | タスク実行環境の準備・クリーンアップ |
| ブランチの作成・マージ・削除 | Feature/タスクブランチの管理 |

#### 編集禁止

| 対象 | 理由 |
|------|------|
| `tasks.yaml` | 静的定義（設計時に確定） |
| `spec.md`（feature, タスク） | 仕様の変更禁止 |
| `report.md` | ワーカー専用 |
| ソースコード | ワーカー専用 |

### 禁止操作

| 操作 | 理由 |
|------|------|
| タスクの実装を行う | ワーカーの責務 |
| `report.md` を作成・編集する | ワーカー専用 |
| 依存関係を無視したディスパッチ | タスク間の整合性を破壊 |

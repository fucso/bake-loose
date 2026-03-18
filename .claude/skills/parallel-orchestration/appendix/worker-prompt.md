# ワーカー用プロンプトテンプレート

オーケストレーターがワーカーを起動する際に使用するプロンプト。

---

## 起動コマンド

```bash
cd {worktree_path} && claude -p "{prompt}" > {log_path} 2>&1 &
```

---

## プロンプトテンプレート

```
以下のタスクを実行してください。

## タスク情報

- タスク仕様: {task_spec_path}
- Feature 仕様: {feature_spec_path}
- ワーカーガイド: .claude/skills/parallel-orchestration/references/worker-behavior.md

## 環境変数

- WORKTREE_PATH: {worktree_path}
- MAIN_REPO_PATH: {main_repo_path}
- DOCKER_WORKTREE_PATH: {docker_worktree_path}

## 実行手順

1. AGENTS.md を読み込む
2. ワーカーガイド（worker-behavior.md）を読み込む
3. タスク仕様（spec.md）を読み込む
4. 依存タスクの report.md があれば参照する
5. 対象レイヤーのコーディングルールを参照する（.claude/rules/ 配下）
6. 実装を行う
7. ビルド・テストを実行する
8. 実装コードをコミットする
9. report.md を作成し、別コミットとしてコミットする

## 制約

- report.md 以外の .agents/ 配下のファイルは編集しない
- タスク仕様に記載された範囲のみを実装する
- Docker Compose 環境は共有されている（他ワーカーと同時実行の可能性あり）

## 完了条件

タスクディレクトリに report.md を作成・コミットすることで完了を通知する。
```

---

## 変数一覧

| 変数 | 説明 | 例 |
|------|------|-----|
| `{worktree_path}` | ワーカーが作業する worktree ディレクトリ | `/path/to/project/.agents/worktrees/20260209-parallel_orchestration_01-skill-definition` |
| `{main_repo_path}` | メインリポジトリのパス | `/path/to/project` |
| `{docker_worktree_path}` | Docker コンテナ内での worktree パス | `/worktrees/20260209-parallel_orchestration_01-skill-definition` |
| `{task_spec_path}` | タスク仕様のパス | `.agents/features/20260209-parallel_orchestration/tasks/01-skill-definition/spec.md` |
| `{feature_spec_path}` | Feature 仕様のパス | `.agents/features/20260209-parallel_orchestration/spec.md` |
| `{log_path}` | ワーカー出力のログファイル | `.agents/features/20260209-parallel_orchestration/tasks/01-skill-definition/worker_output.log` |

---

## オーケストレーターでの使用例

```bash
# 変数の設定
FEATURE_ID="20260209-parallel_orchestration"
TASK_ID="01-skill-definition"
MAIN_REPO_PATH="/path/to/project"
WORKTREE_PATH="${MAIN_REPO_PATH}/.agents/worktrees/${FEATURE_ID}_${TASK_ID}"
DOCKER_WORKTREE_PATH="/worktrees/${FEATURE_ID}_${TASK_ID}"
TASK_SPEC_PATH=".agents/features/${FEATURE_ID}/tasks/${TASK_ID}/spec.md"
FEATURE_SPEC_PATH=".agents/features/${FEATURE_ID}/spec.md"
LOG_PATH=".agents/features/${FEATURE_ID}/tasks/${TASK_ID}/worker_output.log"

# プロンプトの構築（ヒアドキュメント）
PROMPT=$(cat <<EOF
以下のタスクを実行してください。

## タスク情報

- タスク仕様: ${TASK_SPEC_PATH}
- Feature 仕様: ${FEATURE_SPEC_PATH}
- ワーカーガイド: .claude/skills/parallel-orchestration/references/worker-behavior.md

## 環境変数

- WORKTREE_PATH: ${WORKTREE_PATH}
- MAIN_REPO_PATH: ${MAIN_REPO_PATH}
- DOCKER_WORKTREE_PATH: ${DOCKER_WORKTREE_PATH}

## 実行手順

1. AGENTS.md を読み込む
2. ワーカーガイド（worker-behavior.md）を読み込む
3. タスク仕様（spec.md）を読み込む
4. 依存タスクの report.md があれば参照する
5. 対象レイヤーのコーディングルールを参照する（.claude/rules/ 配下）
6. 実装を行う
7. ビルド・テストを実行する
8. 実装コードをコミットする
9. report.md を作成し、別コミットとしてコミットする

## 制約

- report.md 以外の .agents/ 配下のファイルは編集しない
- タスク仕様に記載された範囲のみを実装する
- Docker Compose 環境は共有されている（他ワーカーと同時実行の可能性あり）

## 完了条件

タスクディレクトリに report.md を作成・コミットすることで完了を通知する。
EOF
)

# ワーカーの起動
cd "${WORKTREE_PATH}" && claude -p "${PROMPT}" > "${LOG_PATH}" 2>&1 &
```

# ワーカーの振る舞い

ワーカーエージェントの役割、実行手順、制約事項。

---

## 概要

ワーカーは worktree 内で起動され、指定されたタスクを自律的に実行する。
オーケストレーターとの通信は `report.md` ファイルの作成のみで行う。

### 責務

- タスク仕様に従った実装
- ビルド・テストの実行
- 完了レポートの作成

### worktree 環境の理解

ワーカーは git worktree 内で動作する。以下の変数を理解しておく必要がある:

| 変数 | 説明 | 例 |
|------|------|-----|
| `WORKTREE_PATH` | ワーカーが作業する worktree ディレクトリ | `/path/to/project/.worktrees/task_20260209-xxx_01-task` |
| `MAIN_REPO_PATH` | メインリポジトリのパス | `/path/to/project` |
| `DOCKER_WORKTREE_PATH` | Docker コンテナ内での worktree パス | `/worktrees/task_20260209-xxx_01-task` |

**注意**: Docker Compose 環境はメインリポジトリで起動されており、worktree ディレクトリは以下のようにマウントされている:

```yaml
# compose.yaml での設定
volumes:
  - ./.worktrees:/worktrees
```

これにより、ホストの `.worktrees/task_xxx` がコンテナ内の `/worktrees/task_xxx` にマッピングされる。

**注意**: worktree は `background-developing-with-worktree` skill によって `.worktrees/` ディレクトリに作成される。各 worktree には `.worktree.env` が自動生成され、パス情報が記録されている。

---

## 実行手順

ワーカーは以下の順序でタスクを実行する:

### 1. タスク仕様の読み込み

- `{task_dir}/spec.md` を読み込む
- 目的、変更対象、設計詳細、完了条件を理解する

### 2. 依存タスクの参照

- 依存タスクの `report.md` があれば参照
- 「次タスクへの申し送り」セクションを確認
- 依存タスクで作成されたコードを把握

### 3. コーディングルールの参照

- 対象レイヤーの `.claude/rules/` を参照
- プロジェクトの設計思想に従う

### 4. 実装

- タスク仕様に従って実装
- 完了条件を意識しながら進める

### 5. ビルド・テスト

- Docker Compose 環境でビルド・テストを実行
- 失敗した場合は修正して再実行

### 6. 実装コードのコミット

- 実装コードをコミット
- 適切なコミットメッセージを記載
- **worktree ディレクトリを指定して git コマンドを実行**

```bash
# worktree ディレクトリを指定して git add
git -C ${WORKTREE_PATH} add <files>

# worktree ディレクトリを指定して git commit
git -C ${WORKTREE_PATH} commit -m "feat({feature-id}): 実装内容"
```

### 7. レポート作成・コミット

- `report.md` を作成
- [report.md テンプレート](../appendix/report-md.md) に従う
- **report.md を別コミットとしてコミット**（実装コードとは分離）

```bash
git -C ${WORKTREE_PATH} add .agents/features/{feature-id}/tasks/{task-id}/report.md
git -C ${WORKTREE_PATH} commit -m "report({feature-id}): {タスク名} 完了レポート"
```

**重要**: report.md のコミットがオーケストレーターへの完了通知となる。

---

## Docker Compose 環境

Docker Compose 環境は全ワーカーで共有されている。
`compose.yaml` はメインリポジトリにあり、worktree ディレクトリはコンテナにマウントされている。

### コンテナの競合防止

- 同時に複数のワーカーが `docker compose exec` を実行する可能性がある
- コンテナ内の操作は互いに影響しないようにする
- ポートの競合には注意（通常は問題ない）

### worktree からの docker compose 実行

worktree ディレクトリから docker compose を実行する場合、`-f` オプションでメインリポジトリの `compose.yaml` を指定する必要がある。

#### コマンドの基本形式

```bash
docker compose -f ${MAIN_REPO_PATH}/compose.yaml exec <service> bash -c "cd ${DOCKER_WORKTREE_PATH}/<subdir> && <command>"
```

worktree のコードを対象にコマンドを実行するには必ず以下の2ステップを行う:

1. **カレントディレクトリを Docker 内の worktree ディレクトリに移動**
2. **テストなどのコマンドを実行**

### ビルド・テストコマンド

worktree で実装したコードに対してビルド・テストを実行するには、コンテナ内で worktree ディレクトリに移動してからコマンドを実行する:

```bash
# バックエンドのビルド（worktree のコードを対象）
docker compose -f ${MAIN_REPO_PATH}/compose.yaml exec backend bash -c "cd ${DOCKER_WORKTREE_PATH}/backend && cargo build"

# バックエンドのテスト（worktree のコードを対象）
docker compose -f ${MAIN_REPO_PATH}/compose.yaml exec backend bash -c "cd ${DOCKER_WORKTREE_PATH}/backend && cargo test"

# フォーマット（worktree のコードを対象）
docker compose -f ${MAIN_REPO_PATH}/compose.yaml exec backend bash -c "cd ${DOCKER_WORKTREE_PATH}/backend && cargo fmt"

# Lint（worktree のコードを対象）
docker compose -f ${MAIN_REPO_PATH}/compose.yaml exec backend bash -c "cd ${DOCKER_WORKTREE_PATH}/backend && cargo clippy --all-targets -- -D warnings"
```

**注意**: `DOCKER_WORKTREE_PATH` は Docker コンテナ内でのパスであり、ホストの `WORKTREE_PATH` とは異なる。

---

## worktree の扱い

ワーカーは worktree 内で起動され、コマンド実行時には worktree ディレクトリを明示的に指定する必要がある。

### ワーカーの視点

- 作業ディレクトリは worktree パス（`WORKTREE_PATH`）
- git 操作は `-C` オプションで worktree を指定
- docker compose は `-f` オプションでメインリポジトリの compose.yaml を指定
- ブランチの作成・切り替えは不要（既に設定済み）

---

## 完了通知

ワーカーはタスク完了時に `report.md` を作成・コミットすることでオーケストレーターに完了を通知する。

### report.md の配置

worktree 内のタスクディレクトリに作成:

```
{WORKTREE_PATH}/.agents/features/{feature-id}/tasks/{task-id}/report.md
```

### コミット順序

1. **実装コードをコミット**
2. **report.md を別コミットとしてコミット**

この順序を守ることで:
- 実装の履歴とレポートの履歴が分離される
- オーケストレーターは report.md のコミットで完了を検知できる

### オーケストレーターの検知

オーケストレーターは git を監視して report.md のコミットを検知する。
コミットが存在した時点でタスク完了と判断する。
ファイルシステムではなく git を監視することで、コミット前の作業途中状態を誤検知しない。

---

## エラー時の対応

### ビルドエラー

1. エラー内容を確認
2. コードを修正
3. 再ビルド
4. 成功するまで繰り返す

### テストエラー

1. 失敗したテストを確認
2. コードまたはテストを修正
3. 再テスト
4. 成功するまで繰り返す

### 解決できないエラー

1. `report.md` にエラー内容を記載
2. 状況を詳細に説明
3. オーケストレーターが `failed` として処理

---

## 制約事項

### 編集権限

#### 編集可能

| 対象 | 説明 |
|------|------|
| ソースコード | タスク仕様で指定されたファイル |
| `{task_dir}/report.md` | タスク完了時に作成 |

#### 編集禁止

| 対象 | 理由 |
|------|------|
| `active.yaml` | オーケストレーター専用 |
| `tasks.yaml` | 静的定義（設計時に確定） |
| `status.yaml` | オーケストレーター専用 |
| feature, タスクの `spec.md` | タスク定義の変更禁止 |
| 他タスクの `report.md` | 他ワーカーの成果物 |

### 禁止操作

| 操作 | 理由 |
|------|------|
| `git checkout` でブランチを切り替える | worktree のブランチは固定 |
| `git worktree` コマンドを使う | オーケストレーター専用 |
| 親ディレクトリ（`.worktrees/` の外）を参照する | 他 worktree との競合防止 |
| オーケストレーション状態ファイルを編集する | 役割分離の原則 |

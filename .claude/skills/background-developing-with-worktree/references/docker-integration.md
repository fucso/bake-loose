# Docker Compose Integration

Worktree 内のコードに対して Docker Compose でビルド・テストを実行する方法。

## 前提条件

### Docker Compose のマウント設定

`compose.yaml` に worktree ディレクトリのマウント設定が必要です:

```yaml
services:
  backend:
    volumes:
      - ./.worktrees:/worktrees  # worktree ディレクトリをマウント
      - ./backend:/app           # メインリポジトリもマウント
```

この設定により:
- ホストの `.worktrees/` が Docker 内の `/worktrees/` にマッピングされる
- 各 worktree は `/worktrees/<worktree-name>/` としてアクセス可能

## 設定ファイル（.worktree.env）

Docker 統合スクリプトは `.worktree.env` ファイルから設定を自動的に読み込みます:

```bash
# .worktree.env（setup-worktree.sh により自動生成）
WORKTREE_PATH="/Users/user/project/.worktrees/feature_new-feature"
MAIN_REPO_PATH="/Users/user/project"
DOCKER_WORKTREE_PATH="/worktrees/feature_new-feature"
BRANCH_NAME="feature/new-feature"
FEATURE_ID="feature/new-feature"
```

**重要:** スクリプトは worktree ディレクトリから実行する必要があります。`.worktree.env` が自動的に検出されます。

## スクリプトの使用方法

### docker-exec.sh (汎用ラッパー)

Docker Compose exec のラッパースクリプト。任意のコマンドを worktree 内で実行できます。

**基本形式:**
```bash
docker-exec.sh <service> <command> [subdir]
```

**パラメータ:**
- `service`: サービス名（backend, frontend など）
- `command`: 実行するコマンド
- `subdir`: (オプション) worktree 内のサブディレクトリ

**例:**
```bash
# backend サービスで cargo build を実行
docker-exec.sh backend "cargo build"

# backend サービスの backend/ サブディレクトリで cargo test を実行
docker-exec.sh backend "cargo test" backend

# 環境変数の確認
docker-exec.sh backend "pwd"
```

**仕組み:**
1. `DOCKER_WORKTREE_PATH` と `subdir` を結合して作業ディレクトリを決定
2. Docker コンテナ内でそのディレクトリに `cd` してからコマンド実行
3. `compose.yaml` のパスは `MAIN_REPO_PATH` から自動解決

### build.sh

バックエンドのビルドを実行します。

**使用例:**
```bash
# worktree ディレクトリから実行（.worktree.env から設定を自動読み込み）
scripts/build.sh
```

**内部実装:**
```bash
docker-exec.sh backend "cargo build"
```

### test.sh

バックエンドのテストを実行します。

**使用例:**
```bash
# worktree ディレクトリから実行（.worktree.env から設定を自動読み込み）
scripts/test.sh
```

**内部実装:**
```bash
docker-exec.sh backend "cargo test"
```

### lint.sh

バックエンドの Lint を実行します。

**使用例:**
```bash
# worktree ディレクトリから実行（.worktree.env から設定を自動読み込み）
scripts/lint.sh
```

**内部実装:**
```bash
docker-exec.sh backend "cargo clippy -- -D warnings"
```

## 完全なワークフロー例

### 新機能開発

```bash
# 1. Worktree セットアップ（.worktree.env が自動生成される）
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/api-endpoint main)

# 2. 作業ディレクトリに移動
cd ${WORKTREE_PATH}

# 3. 実装
# ... コードを編集 ...

# 4. ビルド（.worktree.env から設定を自動読み込み）
scripts/build.sh

# 5. テスト（.worktree.env から設定を自動読み込み）
scripts/test.sh

# 6. Lint（.worktree.env から設定を自動読み込み）
scripts/lint.sh

# 7. コミット（.worktree.env から設定を自動読み込み）
scripts/commit.sh feat "Add new API endpoint" backend/src/

# 8. メインリポジトリに戻る
cd $(git rev-parse --show-toplevel)

# 9. マージ
git merge feature/api-endpoint

# 10. Worktree クローズ
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

## トラブルシューティング

### .worktree.env が見つからない

```
Error: .worktree.env not found
```

**原因:**
- worktree ディレクトリ外からスクリプトを実行している
- `.worktree.env` が削除されている

**対処法:**
```bash
# worktree ディレクトリに移動してから実行
cd /path/to/.worktrees/feature_name
scripts/build.sh
```

または `.worktree.env` を再作成（setup-worktree.sh のコードを参照）

### Docker コンテナが起動していない

```
Error: No container found for service
```

**対処法:**
```bash
cd ${MAIN_REPO_PATH}
docker compose up -d
```

### Worktree がマウントされていない

```
bash: cd: /worktrees/...: No such file or directory
```

**対処法:**
1. `compose.yaml` にマウント設定を追加
2. Docker Compose を再起動:
   ```bash
   docker compose down
   docker compose up -d
   ```

### パスが正しくない

**確認方法:**
```bash
# ホストのパス確認
echo ${WORKTREE_PATH}
ls ${WORKTREE_PATH}

# Docker 内のパス確認
docker compose exec backend ls /worktrees
```

## パスのマッピング

理解しやすくするために、パスのマッピングを図示します:

```
Host                                  Docker Container
────────────────────────────────────  ──────────────────────────────
/Users/user/project/                  /app/
  ├── .worktrees/                     /worktrees/
  │   └── feature_new-feature/        └── feature_new-feature/
  │       └── backend/                    └── backend/
  │           └── src/                        └── src/
  └── compose.yaml
```

スクリプトは:
1. ホスト側で `WORKTREE_PATH` を使用してファイル操作
2. Docker 側で `DOCKER_WORKTREE_PATH` を使用してコマンド実行
3. `MAIN_REPO_PATH` を使用して `compose.yaml` の位置を特定

## カスタムコマンドの実行

独自のコマンドを実行したい場合は、`docker-exec.sh` を直接使用します:

```bash
# フォーマットの実行
docker-exec.sh backend "cargo fmt" backend

# 特定のテストの実行
docker-exec.sh backend "cargo test test_name" backend

# 依存関係の更新
docker-exec.sh backend "cargo update" backend

# シェルを起動
docker-exec.sh backend "bash"
```

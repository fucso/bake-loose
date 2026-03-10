# Complete Workflow Examples

実際の開発シーンでの完全なワークフロー例。

## Table of Contents

- [新機能開発](#新機能開発)
- [並列開発](#並列開発)
- [実験的な変更](#実験的な変更)
- [バグ修正](#バグ修正)

---

## 新機能開発

ユーザー認証機能を実装する例。

```bash
# 1. Worktree セットアップ（.worktree.env が自動生成される）
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/user-auth main)
echo "Created worktree at: ${WORKTREE_PATH}"

# 2. 作業ディレクトリに移動
cd ${WORKTREE_PATH}

# 3. 実装
# ... コードを編集 ...

# 4. ビルド・テスト（.worktree.env から設定を自動読み込み）
scripts/build.sh
scripts/test.sh

# 5. コミット（.worktree.env から設定を自動読み込み）
scripts/commit.sh feat "Add user authentication" src/

# 6. メインリポジトリに戻る
cd $(git rev-parse --show-toplevel)

# 7. マージ
git merge feature/user-auth

# 8. Worktree クローズ
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

---

## 並列開発

複数の機能を同時に開発する例。

```bash
# タスク1の worktree
WORKTREE_A=$(scripts/setup-worktree.sh feature/api-endpoint main)

# タスク2の worktree
WORKTREE_B=$(scripts/setup-worktree.sh feature/ui-component main)

# それぞれの worktree で独立して作業
cd ${WORKTREE_A}
# ... API エンドポイントを実装 ...
scripts/build.sh
scripts/test.sh
scripts/commit.sh feat "Add API endpoint" backend/src/

cd ${WORKTREE_B}
# ... UI コンポーネントを実装 ...
scripts/build.sh
scripts/test.sh
scripts/commit.sh feat "Add UI component" frontend/src/

# メインリポジトリに戻ってマージ
cd $(git rev-parse --show-toplevel)
git merge feature/api-endpoint
git merge feature/ui-component

# クリーンアップ
scripts/close-worktree.sh ${WORKTREE_A} --delete-branch
scripts/close-worktree.sh ${WORKTREE_B} --delete-branch
```

---

## 実験的な変更

新しいアルゴリズムを試す例。うまくいかなければブランチごと削除。

```bash
# 実験用 worktree を作成
WORKTREE_PATH=$(scripts/setup-worktree.sh experiment/new-algorithm main)

# 作業
cd ${WORKTREE_PATH}
# ... 実験的なコードを編集 ...

scripts/build.sh
scripts/test.sh

# 結果を確認
# ...

# 【パターン1】うまくいかなかった場合 → ブランチごと削除
cd $(git rev-parse --show-toplevel)
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch

# 【パターン2】うまくいった場合 → マージしてから削除
cd $(git rev-parse --show-toplevel)
git merge experiment/new-algorithm
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

---

## バグ修正

本番環境で発見されたバグを修正する例。

```bash
# 1. バグ修正用 worktree をセットアップ
WORKTREE_PATH=$(scripts/setup-worktree.sh fix/login-validation main)

# 2. 作業ディレクトリに移動
cd ${WORKTREE_PATH}

# 3. バグを修正
# ... コードを修正 ...

# 4. テスト（バグが直っていることを確認）
scripts/test.sh

# 5. コミット
scripts/commit.sh fix "Fix login validation bug" src/auth/

# 6. メインリポジトリに戻る
cd $(git rev-parse --show-toplevel)

# 7. マージ
git merge fix/login-validation

# 8. クリーンアップ
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch

# 9. 本番デプロイ
# ... デプロイ処理 ...
```

---

## Docker を使った開発

Docker Compose でビルド・テストを実行する完全な例。

```bash
# 1. Worktree セットアップ
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/api-endpoint main)

# 2. 作業ディレクトリに移動
cd ${WORKTREE_PATH}

# 3. 実装
# ... コードを編集 ...

# 4. Docker でビルド（.worktree.env から設定を自動読み込み）
scripts/build.sh

# 5. Docker でテスト（.worktree.env から設定を自動読み込み）
scripts/test.sh

# 6. Lint（.worktree.env から設定を自動読み込み）
scripts/lint.sh

# 7. すべて成功したらコミット
scripts/commit.sh feat "Add new API endpoint" backend/src/

# 8. メインリポジトリに戻る
cd $(git rev-parse --show-toplevel)

# 9. マージ
git merge feature/api-endpoint

# 10. クリーンアップ
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

---

## 複数コミットのワークフロー

段階的に実装して複数回コミットする例。

```bash
# 1. Worktree セットアップ
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/complex-feature main)
cd ${WORKTREE_PATH}

# 2. Phase 1: モデル実装
# ... モデルを実装 ...
scripts/build.sh
scripts/test.sh
scripts/commit.sh feat "Add domain model" src/domain/

# 3. Phase 2: ユースケース実装
# ... ユースケースを実装 ...
scripts/build.sh
scripts/test.sh
scripts/commit.sh feat "Add use case" src/use_case/

# 4. Phase 3: API エンドポイント実装
# ... API を実装 ...
scripts/build.sh
scripts/test.sh
scripts/commit.sh feat "Add API endpoint" src/presentation/

# 5. 完了したらマージ
cd $(git rev-parse --show-toplevel)
git merge feature/complex-feature
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

---

## トラブル発生時の対応

ビルドエラーやテスト失敗が発生した場合の対処例。

```bash
# 1. Worktree セットアップ
WORKTREE_PATH=$(scripts/setup-worktree.sh feature/new-feature main)
cd ${WORKTREE_PATH}

# 2. 実装
# ... コードを編集 ...

# 3. ビルド実行
scripts/build.sh
# → エラー発生！

# 4. エラーログを確認
# docker-exec.sh backend "cargo build" のログを見る

# 5. エラーを修正
# ... コードを修正 ...

# 6. 再ビルド
scripts/build.sh
# → 成功！

# 7. テスト実行
scripts/test.sh
# → 1つのテストが失敗！

# 8. テストを修正または実装を修正
# ...

# 9. 再テスト
scripts/test.sh
# → すべて成功！

# 10. コミット
scripts/commit.sh feat "Add new feature" src/

# 11. 完了処理
cd $(git rev-parse --show-toplevel)
git merge feature/new-feature
scripts/close-worktree.sh ${WORKTREE_PATH} --delete-branch
```

---

## Tips

### スクリプトパスの簡略化

worktree 内で頻繁にスクリプトを実行する場合、alias を設定すると便利です:

```bash
# .worktree.env にエイリアスを追加
cat >> .worktree.env << 'EOF'

# Aliases for convenience
alias wbuild='scripts/build.sh'
alias wtest='scripts/test.sh'
alias wlint='scripts/lint.sh'
alias wcommit='scripts/commit.sh'
EOF

# .worktree.env を再読み込み
source .worktree.env

# 簡単にコマンド実行
wbuild
wtest
wcommit feat "Add feature" src/
```

### 作業の一時保存

作業途中で別の緊急タスクが入った場合:

```bash
# 現在の worktree で作業中
cd /path/to/.worktrees/feature_current

# 変更を一時保存
git stash save "WIP: current work"

# 別の worktree で緊急タスク
URGENT_WORKTREE=$(scripts/setup-worktree.sh fix/urgent-bug main)
cd ${URGENT_WORKTREE}
# ... 緊急対応 ...
scripts/commit.sh fix "Fix urgent bug" src/
cd $(git rev-parse --show-toplevel)
git merge fix/urgent-bug
scripts/close-worktree.sh ${URGENT_WORKTREE} --delete-branch

# 元の worktree に戻る
cd /path/to/.worktrees/feature_current
git stash pop
# ... 作業を続ける ...
```

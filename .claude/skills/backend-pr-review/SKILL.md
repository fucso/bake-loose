---
name: backend-pr-review
description: |
  Rust backendのPRレビューを実施するスキル。レイヤードアーキテクチャの原則遵守、Rust実装規範、テスト網羅性を検証する。
  使用タイミング: (1) GitHub PRのレビュー依頼、(2) 実装コードのアーキテクチャ検証、(3) rulesへの準拠確認、(4) feature仕様との乖離チェック
---

# Backend PR Review

Rust backendのPRをレビューするためのワークフロー。

## レビューワークフロー

### 1. コンテキスト収集

1. **AGENTS.md を読む** - プロジェクト構造と設計思想を理解
2. **PR情報を取得** - `mcp__github__pull_request_read` で差分と変更ファイル一覧を取得
3. **feature仕様を確認**（指定された場合）- `.agents/features/{feature-name}/` 配下の仕様を読む

### 2. レイヤー別レビュー

依存方向に沿って内側から順にレビュー:

```
domain → ports → repository → use_case → presentation
```

各レイヤーの詳細なレビュー観点は [references/layer-checklist.md](references/layer-checklist.md) を参照。

### 3. 必須チェック項目

#### Rust実装規範

- `clippy` 警告がないこと
- `rustfmt` でフォーマット済みであること
- エラー処理が適切（`unwrap()` の乱用なし）
- 借用と所有権が適切
- `async`/`await` の正しい使用

#### rules準拠（必須）

`.claude/rules/` 配下の全ルールに準拠しているか確認:

| ルールファイル | 主なチェック項目 |
|--------------|----------------|
| domain.md | 外部クレート禁止、validate/execute/runパターン、エラーは種類のみ |
| ports.md | domain層のみ依存、外部クレートなし |
| repository.md | PgExecutor使用、UPSERTパターン、ビジネスロジック禁止 |
| use-case.md | UnitOfWork経由、begin/commit/rollback、DB検証が先 |
| presentation.md | ラッパー型使用、UserFacingError実装 |
| testing.md | MockUnitOfWork共通化、GraphQLテストヘルパー使用 |

#### feature仕様との照合（指定時）

feature仕様が渡された場合、以下を確認:
- 機能要件が全て実装されているか
- 仕様で定義されたエラーケースが全て処理されているか
- テストケースが仕様を網羅しているか
- GraphQLスキーマが仕様と一致しているか

### 4. テスト網羅性確認

仕様で定義されたテストケースが全て実装されているか表形式で確認:

```markdown
| 仕様のテストケース | 実装状況 |
|------------------|---------|
| test_xxx | ✅ / ❌ |
```

### 5. レビュー結果出力

以下の形式で出力:

```markdown
## PR #{number} レビュー結果

### 総評
{全体の評価}

### ✅ 良い点
{箇条書き}

### ⚠️ 指摘事項
{指摘があれば記載、なければ「なし」}

### 📋 rules準拠チェック
| ルール | 準拠状況 |
|-------|---------|
| {rule} | ✅ / ❌ |

### 🔍 テスト網羅性
{テーブル形式}

### 結論
**レビュー結果: Approve / Request Changes**
```

## コマンド例

```bash
# PRレビュー
docker compose exec backend bash -c "cargo clippy -- -D warnings"
docker compose exec backend bash -c "cargo fmt -- --check"
docker compose exec backend bash -c "cargo test"
```

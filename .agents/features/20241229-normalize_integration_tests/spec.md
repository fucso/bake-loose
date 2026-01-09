# Feature: normalize-integration-tests

## 概要

async-graphql のスキーマ直接実行と sqlx::test マクロを組み合わせ、リクエストレベルテストの基盤を正規化する。

## 元の要件

> async-graphql-axum と sqlx::test マクロを組み合わせたリクエストレベルテストの正規化

---

## 要件分析

### 機能要件

- **スキーマ直接実行**: `schema.execute()` を使用し、HTTP レイヤーをスキップした GraphQL テストを実現
- **テスト分離**: `sqlx::test` マクロによる自動トランザクション管理でテスト間の干渉を防止
- **フィクスチャ**: テストデータを `fixtures/` ディレクトリから SQL で投入
- **完全置き換え**: 既存の `serial_test` + 手動クリーンアップを廃止

### 非機能要件

- **高速化**: HTTP レイヤースキップ + 並列実行によるテスト速度向上
- **保守性**: ボイラープレートコード削減、テストデータ管理の一元化

---

## 影響範囲

| レイヤー | 影響 | 変更概要 |
|----------|------|----------|
| domain   | なし | - |
| ports    | なし | - |
| use_case | なし | - |
| repository | なし | - |
| presentation | なし | - |
| migration | なし | - |
| **tests** | あり | テスト基盤の全面刷新 |
| **Cargo.toml** | あり | dev-dependencies 変更 |

---

## タスク分解

### 分解方針

テスト基盤の変更は段階的に行う必要がある:
1. まず新しいテスト基盤（スキーマビルダー、フィクスチャ）を整備
2. 次に既存テストを新基盤に移行

依存関係がシンプルなため、2タスクに分割。

### タスク一覧

| # | タスク | ディレクトリ | 依存 |
|---|--------|--------------|------|
| 01 | テスト基盤のセットアップ | [01-setup-sqlx-test/](./tasks/01-setup-sqlx-test/) | - |
| 02 | 既存テストの移行 | [02-migrate-existing-tests/](./tasks/02-migrate-existing-tests/) | 01 |

### 実装順序

```mermaid
flowchart LR
    01[01-setup-sqlx-test] --> 02[02-migrate-existing-tests]
```

---

## 前提条件

- Docker Compose 環境でテスト実行可能であること
- PostgreSQL テスト用データベースが利用可能であること

## オープンクエスチョン

なし

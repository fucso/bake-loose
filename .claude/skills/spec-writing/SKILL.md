---
name: spec-writing
description: |
  仕様ドキュメント (spec.md) や GitHub issue を記載する際のガイドライン。
  使用タイミング: (1) 設計コマンドでの spec.md 作成、(2) feature/task 仕様の作成、(3) GitHub issue の作成・編集
---

# 仕様ドキュメント記載スキル

## 概要

仕様ドキュメント (spec.md) や GitHub issue を記載する際のガイドライン。
設計コマンド、feature 仕様作成、GitHub issue 作成など、あらゆるコンテキストで適用する。

---

## 基本方針

**実装者が「何をすべきか」を理解できる粒度で記載する。**

具体的なコードは実装時の状況（既存コードとの整合性、リファクタリングの機会など）に応じて決定するため、仕様には含めない。

---

## 記載すべきこと

| 項目 | 説明 | 例 |
|------|------|-----|
| ロジックの流れ | 処理の順序を日本語で | 「入力を検証し、エンティティを生成し、永続化する」 |
| バリデーションルール | 条件と理由 | 「Steps が空でないこと（空の Trial は意味をなさないため）」 |
| データ構造の概念 | フィールド名と用途 | 「Command は project_id, memo, steps を持つ」 |
| エラーケースと対応 | どのような場合にどうなるか | 「key が空の場合は EmptyKey エラーを返す」 |
| 完了条件 | 何をもって完了とするか | チェックリスト形式で |

---

## 記載しないこと

| 項目 | 理由 |
|------|------|
| 完全なコード実装 | 実装時の柔軟性を奪う |
| 具体的な型シグネチャや関数定義 | 既存コードとの整合性は実装時に判断 |
| テストの実装コード | テストケースの意図のみ記載 |
| インポート文やモジュール構成の詳細 | 実装時に決定 |

---

## 記載例

### 良い例

```markdown
## validate 関数

以下を検証する:

- Steps が空でないこと
  - 空の場合: EmptySteps エラー
- 各 Step 内の Parameter を検証
  - KeyValue 型: key が空でないこと、Text 値の場合は値も空でないこと
  - Text 型: value が空でないこと
  - TimePoint 型: note が空でないこと
- エラー時は step_index と parameter_index を含めて返す
```

### 避けるべき例

```markdown
## validate 関数

\`\`\`rust
pub fn validate(command: &Command) -> Result<(), Error> {
    if command.steps.is_empty() {
        return Err(Error::EmptySteps);
    }

    for (step_idx, step) in command.steps.iter().enumerate() {
        for (param_idx, param) in step.parameters.iter().enumerate() {
            if let Err(reason) = validate_parameter(&param.content) {
                return Err(Error::InvalidParameter {
                    step_index: step_idx,
                    parameter_index: param_idx,
                    reason,
                });
            }
        }
    }
    Ok(())
}
\`\`\`
```

---

## テストケースの記載方法

### 記載すべきこと

- テスト名（命名規則に従う）
- テストの意図（何を確認するか）

### 記載しないこと

- テストの実装コード
- アサーションの詳細

### 例

```markdown
## テストケース

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_with_single_step` | 1 つの Step で Trial を作成できる |
| `test_step_positions_are_sequential` | Step の position が 0 から順番に設定される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_steps_empty` | Steps が空の場合 EmptySteps エラー |
| `test_error_contains_step_and_parameter_index` | エラーに位置情報が含まれる |
```

---

## フォーマットテンプレート

- [Feature spec.md](./templates/feature-spec.md) - 機能レベルの仕様
- [Task spec.md](./templates/task-spec.md) - タスクレベルの仕様

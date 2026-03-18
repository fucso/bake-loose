# 記載ルール

## 記載すべきこと

| 項目 | 説明 | 例 |
|------|------|-----|
| ロジックの流れ | 処理の順序を日本語で | 「入力を検証し、エンティティを生成し、永続化する」 |
| バリデーションルール | 条件と理由 | 「Steps が空でないこと（空の Trial は意味をなさないため）」 |
| データ構造の概念 | フィールド名と用途 | 「Command は project_id, memo, steps を持つ」 |
| エラーケースと対応 | どのような場合にどうなるか | 「key が空の場合は EmptyKey エラーを返す」 |
| 完了目標 | Feature は振る舞いレベル、Task は実装+テストレベルで定義 | SKILL.md の参照ドキュメントから Feature 目標 / Task 目標を参照 |

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

避けるべきパターンについては SKILL.md の参照ドキュメントからアンチパターンを参照。

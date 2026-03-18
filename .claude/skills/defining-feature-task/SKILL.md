---
name: defining-feature-task
description: |
  仕様ドキュメント (spec.md) や GitHub issue を記載する際のガイドライン。
  使用タイミング: (1) 設計コマンドでの spec.md 作成、(2) feature/task spec.md の作成、(3) GitHub issue の作成・編集
---

# 仕様ドキュメント記載スキル

## 概要

仕様ドキュメント (spec.md) や GitHub issue を記載する際のガイドライン。
設計コマンド、feature/task spec.md の作成、GitHub issue 作成など、あらゆるコンテキストで適用する。

**基本方針:**
- 実装者が「何をすべきか」を理解できる粒度で記載する。具体的なコードは実装時の状況に応じて決定するため、仕様には含めない。
- **Feature および Task の spec.md には必ず完了目標を設定する。** Feature は振る舞いレベル、Task は実装レベルで目標を定義し、テストで検証可能な形にする。目標のない仕様は完了判定ができないため不完全とみなす。

---

## 参照ドキュメント

- [記載ルール](./references/writing-rules.md) - 記載すべきこと・しないことの判断基準と例
- [テストケースの記載方法](./references/test-case-writing.md) - テストケースの記載ルールと例
- [Feature 目標の定義](./references/feature-goal.md) - Feature の振る舞いレベル目標の定義方法
- [Task 目標の定義](./references/task-goal.md) - Task の実装レベル目標の定義方法
- [アンチパターン](./references/anti-patterns.md) - spec 作成全般で避けるべきパターン

## テンプレート

- [Feature spec.md](./templates/feature-spec.md) - 機能レベルの仕様
- [Task spec.md](./templates/task-spec.md) - タスクレベルの仕様

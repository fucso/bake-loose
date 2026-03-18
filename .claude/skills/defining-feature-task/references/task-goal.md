# Task 目標の定義

## 目標の粒度: 実装レベル

Task の目標は **実装の正しさ** で定義する。
そのタスクで行う実装が正しく機能していることを、テストで検証可能な形で記述する。

---

## 目標の構成要素

Task の目標は以下の 2 つで構成する:

### 1. 実装目標

タスクで定義された実装が正しく行えているか。

**良い実装目標の例:**
- `Trial::record_trial` アクションが Steps 付きの Trial を生成できる
- `TrialRepository` が Steps を含む Trial を永続化・取得できる
- `recordTrial` mutation が Steps フィールドを受け付ける

### 2. テスト目標

その実装を検証するテストの追加・修正が正しく行われ、パスするか。

**良いテスト目標の例:**
- `test_record_trial_with_steps` が追加され、パスする
- `test_returns_error_when_steps_empty` が追加され、パスする
- 既存の `test_record_trial` が修正後もパスする

---

## 記述ルール

1. **テストで検証可能にする**: 各目標に対応するテストケースが存在すること
2. **タスクのスコープに閉じる**: 他タスクの成果物に依存する検証は含めない
3. **具体的な振る舞いを書く**: 「正しく動作する」ではなく「〜を入力すると〜が返る」
4. **テストの追加・修正を明示する**: 新規テストか既存テストの修正かを区別する

---

## Feature と Task の目標の関係

```
Feature 目標（振る舞いレベル）
  「createTrial mutation で Steps 付き Trial が作成できる」
    │
    ├── Task 01 目標（実装レベル）
    │   「domain: Trial が Steps を持てる / record_trial が Steps を検証する」
    │   テスト: test_record_trial_with_steps, test_returns_error_when_steps_empty
    │
    ├── Task 02 目標（実装レベル）
    │   「repository: Steps 付き Trial を永続化・取得できる」
    │   テスト: test_save_trial_with_steps, test_find_trial_includes_steps
    │
    └── Task 03 目標（実装レベル）
        「presentation: mutation が steps input を受け付ける」
        テスト: test_create_trial_mutation_with_steps (統合テスト)
```

**全 Task の目標が達成されたとき、Feature の目標が達成される** ことを設計時に確認する。

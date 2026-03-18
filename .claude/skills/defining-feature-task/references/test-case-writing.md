# テストケースの記載方法

## 記載すべきこと

- テスト名（命名規則に従う）
- テストの意図（何を確認するか）

## 記載しないこと

- テストの実装コード
- アサーションの詳細

---

## 例

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

# Testing

テスト実装に関するルール。

## モックオブジェクトの配置

複数のテストファイルで繰り返し利用されるモックオブジェクトは、専用のファイルを作成して再利用する。

```
backend/src/use_case/
├── project/
│   ├── get_project.rs      # テスト内で use_case::test::MockUnitOfWork を使用
│   └── list_projects.rs
└── test/
    └── mock_unit_of_work.rs  # 共通モック
```

**モジュール定義:**

```rust
// src/use_case.rs
pub mod project;

#[cfg(test)]
pub mod test;

// src/use_case/test.rs
pub mod mock_unit_of_work;

pub use mock_unit_of_work::MockUnitOfWork;
```

**使用例:**

```rust
// src/use_case/project/get_project.rs
#[cfg(test)]
mod tests {
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_get_project() {
        let uow = MockUnitOfWork::with_projects(vec![...]);
        // ...
    }
}
```

## アンチパターン

```rust
// ❌ 各テストファイルで同じモックを重複定義
// get_project.rs
struct MockProjectRepository { ... }

// list_projects.rs
struct MockProjectRepository { ... }  // 重複

// ✅ 共通モジュールに集約して再利用
use crate::use_case::test::MockUnitOfWork;
```

## チェックリスト

- [ ] 複数箇所で使うモックは `test/` ディレクトリに集約
- [ ] テスト用モジュールは `#[cfg(test)]` で囲む
- [ ] モックは必要最小限の実装のみ

---
paths: backend/src/use_case/**/*.rs
---

# Use Case Layer

ユースケース層はドメインアクションを組み合わせてビジネスフローを実現するオーケストレーション層。

## 基本原則

**依存先**: domain層、ports層のみ
**禁止**: repository層、presentation層、infrastructure層への直接依存

**やること**:
- ドメインアクションの呼び出し
- UnitOfWork経由でのリポジトリアクセス
- DB問い合わせが必要な検証
- トランザクション境界の管理

**やらないこと**:
- ビジネスロジックの実装
- 直接的なDB操作
- ユーザー向けメッセージの生成
- SQLの記述
- 個別のリポジトリを直接引数として受け取る

## ファイル配置

```
backend/src/use_case/
├── project/
│   ├── create_project.rs
│   └── ...
├── trial/
└── ...
```

**1ユースケース1ファイル**。ファイル名はドメインアクションと同じ名前。

## 実装パターン

```rust
// src/use_case/project/create_project.rs

use crate::domain::actions::project::create_project;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::unit_of_work::UnitOfWork;
use crate::use_case::project::save_project;

#[derive(Debug)]
pub enum Error {
    Domain(create_project::Error),
    DuplicateName,
    Infrastructure(String),
}

pub struct Input {
    pub name: String,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Project, Error> {
    // 1. DB問い合わせが必要な検証（先に行う）
    if uow.project_repository().exists_by_name(&input.name).await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))? {
        return Err(Error::DuplicateName);
    }

    // 2. ドメインアクション実行
    let command = create_project::Command { name: input.name };
    let project = create_project::run(command).map_err(Error::Domain)?;

    // 3. トランザクション開始（書き込み直前に開始する）
    uow.begin().await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_project(uow, &project).await?;

    // 5. コミット
    uow.commit().await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(project)
}
```

**トランザクションの保持は最小限にする**。実際に書き込むのは `save()` のみであるため、
`begin()` はデータ取得とドメインアクションが成功した後、`save()` の直前で呼び出す。
読み取りとドメインアクションは書き込みを伴わないためトランザクション内で実行する必要はない。

`begin()` より前のエラーパスではトランザクションが開始されていないため `rollback()` は呼ばない。
`rollback()` が必要なのは `begin()` 以降（`save()` 失敗時）のみ。

**永続化は必ず集約ごとの save ヘルパー経由で行う**。

| 集約 | ヘルパー | 定義場所 |
|------|----------|----------|
| Project | `save_project(uow, &project)` | `use_case/project.rs` |
| Trial | `save_trial(uow, &trial)` | `use_case/trial.rs` |

ヘルパーは「リポジトリの生存範囲をブロックで閉じ、失敗時に `rollback()` を呼び、
その失敗をログに残す」までを内包する。ユースケース側に書き込み手順を展開しないこと。

```rust
// ❌ リポジトリの一時値が if let 文の終わりまで生存し、rollback() が必ず失敗する
if let Err(e) = uow.project_repository().save(&project).await {
    let _ = uow.rollback().await;  // Arc::try_unwrap が失敗し ROLLBACK が発行されない
    return Err(Error::Infrastructure(format!("{:?}", e)));
}

// ❌ ヘルパーの中身を各ユースケースに展開する（10 箇所それぞれが元の不具合に戻り得る）
let save_result = {
    let repo = uow.project_repository();
    repo.save(&project).await
};
if let Err(e) = save_result { ... }

// ✅ ヘルパー経由。危険な形を書く余地が無い
save_project(uow, &project).await?;
```

**なぜヘルパーに閉じ込めるのか**:
`xxx_repository()` が返すリポジトリはトランザクションの `Arc` を clone して保持する。
edition 2021 では `if let` のスクルーティニー式で作られた一時値は `if let` 文全体の終わりまで
生存するため、本体で `rollback()` を呼ぶ時点でも参照が残る。その結果
`PgUnitOfWork::rollback()` の `Arc::try_unwrap` が失敗し、明示的な `ROLLBACK` が発行されない
（sqlx の `Transaction: Drop` による暗黙のロールバックに依存する状態になる）。
この罠はコンパイルエラーにもならず、注意書きコメントでは必ず drift するため、
「書けないようにする」方向で解決する。

**ロールバック経路のテスト**:
`MockUnitOfWork` は上記の `Arc::try_unwrap` 制約を再現しており、
リポジトリが生存したまま `rollback()` を呼ぶと `Transaction is still in use` で失敗する。
そのためテストでは「呼ばれたこと」ではなく **成功したこと** をアサートする。

```rust
assert_eq!(uow.rollback_count(), 1);
assert_eq!(uow.rollback_success_count(), 1, "ROLLBACK が実際に発行されていない");
assert_eq!(uow.commit_count(), 0);
```

書き込みユースケースには必ず `save()` 失敗経路のテストを用意する（代表数件では不足）。

**読み取り専用のユースケース** では `begin()` は不要:

```rust
pub async fn execute<U: UnitOfWork>(uow: &mut U) -> Result<Vec<Project>, Error> {
    uow.project_repository()
        .find_all(ProjectSort::default())
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}
```

## エラー型

| 種類 | 発生源 |
|------|--------|
| Domain | ドメインアクションのvalidate |
| ビジネスルール | ユースケース層の検証（重複など） |
| Infrastructure | ports実装のエラー |

## アンチパターン

```rust
// ❌ ユースケースでバリデーション
if input.name.is_empty() { return Err(Error::EmptyName); }

// ❌ SQL直接記述
sqlx::query("INSERT INTO ...").execute(pool).await?;

// ❌ 検証の順序が不適切（ドメインアクション後にDB検証）
let project = create_project::run(command)?;
if repository.exists_by_name(&project.name()).await? { ... }

// ❌ リポジトリを個別に引数で受け取る
pub async fn execute(repo: &impl ProjectRepository, input: Input) -> Result<...> { ... }

// ❌ 書き込み操作で begin() を呼ばない
uow.project_repository().save(&project).await?;
uow.commit().await?;  // トランザクションが開始されていない！

// ❌ 読み取り・ドメインアクションをトランザクション内で実行（無駄に保持している）
uow.begin().await?;
let trial = uow.trial_repository().find_by_id(&trial_id).await?;
let trial = add_step::run(trial, command)?;
uow.trial_repository().save(&trial).await?;
uow.commit().await?;

// ❌ begin() 前のエラーパスで rollback() を呼ぶ（トランザクションが存在しない）
let Some(trial) = uow.trial_repository().find_by_id(&trial_id).await? else {
    let _ = uow.rollback().await;
    return Err(Error::NotFound);
};

// ✅ ドメインアクションに委譲、UnitOfWork経由、DB検証を先に、save() 直前に begin()
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<...> { ... }
```

## チェックリスト

- [ ] domain層とports層にのみ依存
- [ ] ドメインアクションを呼び出している（直接ロジック実装していない）
- [ ] UnitOfWork経由で永続化
- [ ] DB検証はドメインアクション実行前
- [ ] 書き込み操作では`begin()`でトランザクション開始
- [ ] `begin()`はデータ取得・ドメインアクションの後、`save()`の直前で呼んでいる（保持は最小限）
- [ ] 成功後に`commit()`を呼んでいる
- [ ] `begin()`以降のエラー時のみ`rollback()`を呼んでいる（`begin()`前は呼ばない）
- [ ] 永続化を`save_project` / `save_trial`ヘルパー経由で行っている（`save()`を直接呼び出していない）
- [ ] `save()`失敗経路のテストがあり、`rollback_success_count()`（呼ばれた回数ではなく成功回数）をアサートしている
- [ ] `RepositoryError::Conflict`を畳み込むときは`entity` / `field`をログに残すか、`field`で分岐している（黙って捨てない）

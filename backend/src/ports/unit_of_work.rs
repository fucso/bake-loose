//! UnitOfWork トレイト
//!
//! 複数リポジトリへのアクセスを一元管理し、トランザクション境界を管理する。

use crate::ports::error::RepositoryError;
use crate::ports::project_repository::ProjectRepository;

/// UnitOfWork トレイト
///
/// トランザクション管理とリポジトリアクセスを提供する。
///
/// ## トランザクションの使用
///
/// 書き込み操作を行う場合は `begin()` でトランザクションを開始し、
/// 成功時は `commit()`、失敗時は `rollback()` を呼び出す。
///
/// ```ignore
/// uow.begin().await?;
/// uow.project_repository().save(&project).await?;
/// uow.commit().await?;
/// ```
///
/// 読み取り専用の場合は `begin()` を呼び出す必要はない。
///
/// ## リポジトリアクセス
///
/// `project_repository()` は呼び出すたびに新しいリポジトリインスタンスを返す。
/// これは Rust の借用ルールに対応するための設計で、パフォーマンスへの影響は軽微。
#[async_trait::async_trait]
pub trait UnitOfWork: Send + Sync {
    /// ProjectRepository の具体型
    type ProjectRepo: ProjectRepository;

    /// ProjectRepository を取得する
    ///
    /// トランザクションが開始されている場合はトランザクション内で操作し、
    /// そうでなければ pool を直接使用する。
    ///
    /// 注: 呼び出すたびに新しいリポジトリインスタンスを返す。
    fn project_repository(&mut self) -> Self::ProjectRepo;

    /// トランザクションを開始する
    ///
    /// 書き込み操作を行う前に呼び出す。
    /// 読み取り専用の場合は呼び出し不要。
    async fn begin(&mut self) -> Result<(), RepositoryError>;

    /// トランザクションをコミットする
    ///
    /// `begin()` で開始したトランザクションを確定する。
    async fn commit(&mut self) -> Result<(), RepositoryError>;

    /// トランザクションをロールバックする
    ///
    /// `begin()` で開始したトランザクションを取り消す。
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}

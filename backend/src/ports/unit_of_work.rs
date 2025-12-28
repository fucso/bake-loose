//! UnitOfWork トレイト
//!
//! 複数リポジトリへのアクセスを一元管理する。
//! 将来的にはトランザクション境界の管理も担う。

use crate::ports::error::RepositoryError;
use crate::ports::project_repository::ProjectRepository;

/// UnitOfWork トレイト
///
/// 複数リポジトリへのアクセスを提供し、トランザクション管理を行う。
#[async_trait::async_trait]
pub trait UnitOfWork: Send + Sync {
    /// ProjectRepository の具体型
    type ProjectRepo: ProjectRepository;

    /// ProjectRepository を取得する
    fn project_repository(&self) -> &Self::ProjectRepo;

    /// トランザクションをコミットする
    ///
    /// 現時点では読み取り専用のため no-op だが、
    /// 書き込み系ユースケース追加時に実装する。
    async fn commit(&mut self) -> Result<(), RepositoryError>;

    /// トランザクションをロールバックする
    ///
    /// 現時点では読み取り専用のため no-op だが、
    /// 書き込み系ユースケース追加時に実装する。
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}

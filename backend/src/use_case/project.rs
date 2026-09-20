//! Project ユースケース
//!
//! プロジェクト関連のユースケースを集約する。

pub mod create_project;
pub mod get_project;
pub mod list_projects;

use crate::domain::models::project::Project;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::{RepositoryError, UnitOfWork};

/// 開始済みトランザクション内で Project を保存し、失敗時はロールバックする
///
/// 書き込みユースケースの「4. 永続化」は必ずこのヘルパーを経由すること。
/// 理由と背景は `use_case::trial::save_trial` と同一。
pub(crate) async fn save_project<U: UnitOfWork>(
    uow: &mut U,
    project: &Project,
) -> Result<(), RepositoryError> {
    let save_result = {
        let repo = uow.project_repository();
        repo.save(project).await
    };

    if let Err(error) = save_result {
        if let Err(rollback_error) = uow.rollback().await {
            log::error!("rollback failed: {:?}", rollback_error);
        }
        return Err(error);
    }

    Ok(())
}

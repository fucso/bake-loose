//! Project ユースケース
//!
//! プロジェクト関連のユースケースを集約する。

pub mod create_project;
pub mod get_project;
pub mod list_projects;

use crate::domain::models::project::Project;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::{RepositoryError, UnitOfWork};
use crate::use_case::rollback_on_error;

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

    rollback_on_error(uow, save_result).await
}

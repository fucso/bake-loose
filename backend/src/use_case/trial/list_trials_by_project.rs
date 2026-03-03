//! list_trials_by_project ユースケース
//!
//! プロジェクトに紐づくトライアル一覧を取得する。

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{TrialSort, UnitOfWork};

#[derive(Debug)]
pub enum Error {
    Infrastructure(String),
}

/// プロジェクトに紐づくトライアル一覧を取得する
///
/// 読み取り専用のためトランザクションは不要。
pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    project_id: &ProjectId,
) -> Result<Vec<Trial>, Error> {
    uow.trial_repository()
        .find_by_project_id(project_id, TrialSort::default())
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

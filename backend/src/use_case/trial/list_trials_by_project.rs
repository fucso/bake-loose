//! list_trials_by_project ユースケース
//!
//! プロジェクトに紐づくTrial一覧を取得する。

use uuid::Uuid;

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;
use crate::ports::trial_repository::{TrialRepository, TrialScope};
use crate::ports::UnitOfWork;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Infrastructure(String),
}

pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    project_id: Uuid,
    scope: TrialScope,
) -> Result<Vec<Trial>, Error> {
    uow.trial_repository()
        .find_all_by_project(&ProjectId(project_id), scope)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_list_trials_by_project_returns_only_matching_project() {
        let mut uow = MockUnitOfWork::default();
        let project_id = ProjectId::new();
        let other_project_id = ProjectId::new();

        let trial1 = Trial::new(project_id.clone(), Some("A".to_string()), None);
        let trial2 = Trial::new(project_id.clone(), Some("B".to_string()), None);
        let other_trial = Trial::new(other_project_id.clone(), Some("Other".to_string()), None);

        uow.trial_repository()
            .save(&trial1, TrialScope::Full)
            .await
            .unwrap();
        uow.trial_repository()
            .save(&trial2, TrialScope::Full)
            .await
            .unwrap();
        uow.trial_repository()
            .save(&other_trial, TrialScope::Full)
            .await
            .unwrap();

        let result = execute(&mut uow, project_id.0, TrialScope::Full).await;

        assert!(result.is_ok());
        let trials = result.unwrap();
        assert_eq!(trials.len(), 2);
        assert!(trials.iter().all(|t| t.project_id() == &project_id));
    }

    #[tokio::test]
    async fn test_list_trials_by_project_empty() {
        let mut uow = MockUnitOfWork::default();
        let result = execute(&mut uow, Uuid::new_v4(), TrialScope::Full).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}

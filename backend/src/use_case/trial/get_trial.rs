//! get_trial ユースケース
//!
//! IDでTrialを取得する。

use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

#[derive(Debug)]
pub enum Error {
    Infrastructure(String),
}

/// IDでTrialを取得する
///
/// 読み取り専用のためトランザクションは不要。
pub async fn execute<U: UnitOfWork>(uow: &mut U, id: &TrialId) -> Result<Option<Trial>, Error> {
    uow.trial_repository()
        .find_by_id(id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_get_trial_returns_specified_trial_from_multiple() {
        let mut uow = MockUnitOfWork::default();
        let project_id = ProjectId::new();
        let target = Trial::new(project_id.clone(), Some("対象".to_string()), None);
        let other = Trial::new(project_id.clone(), Some("別".to_string()), None);
        let target_id = target.id().clone();

        uow.trial_repository().save(&other).await.unwrap();
        uow.trial_repository().save(&target).await.unwrap();

        let result = execute(&mut uow, &target_id).await;

        assert!(result.is_ok());
        let found = result.unwrap().unwrap();
        assert_eq!(found.id(), &target_id);
        assert_eq!(found.name(), Some("対象"));
    }

    #[tokio::test]
    async fn test_get_trial_not_found() {
        let mut uow = MockUnitOfWork::default();
        let result = execute(&mut uow, &TrialId::new()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

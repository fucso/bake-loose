//! add_step ユースケース

use chrono::{DateTime, Utc};

use crate::domain::actions::trial::add_step;
use crate::domain::models::parameter::ParameterContent;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
#[derive(Debug)]
pub struct Input {
    pub trial_id: TrialId,
    pub name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Vec<ParameterInput>,
}

/// パラメーター入力
#[derive(Debug)]
pub struct ParameterInput {
    pub content: ParameterContent,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(add_step::Error),
    TrialNotFound,
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. Trial を取得
    let trial = match uow
        .trial_repository()
        .find_by_id(&input.trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        Some(trial) => trial,
        None => {
            let _ = uow.rollback().await;
            return Err(Error::TrialNotFound);
        }
    };

    // 3. ドメインアクション実行
    let command = add_step::Command {
        name: input.name,
        started_at: input.started_at,
        parameters: input
            .parameters
            .into_iter()
            .map(|p| add_step::ParameterInput { content: p.content })
            .collect(),
    };
    let updated_trial = match add_step::run(trial, command) {
        Ok(t) => t,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.trial_repository().save(&updated_trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(updated_trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::trial::add_step as domain_add_step;
    use crate::domain::models::parameter::{ParameterContent, ParameterValue};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialStatus;
    use crate::use_case::test::MockUnitOfWork;

    fn in_progress_trial() -> Trial {
        Trial::new(ProjectId::new(), None, None)
    }

    fn completed_trial() -> Trial {
        let now = Utc::now();
        Trial::from_raw(
            TrialId::new(),
            ProjectId::new(),
            None,
            None,
            TrialStatus::Completed,
            vec![],
            now,
            now,
        )
    }

    #[tokio::test]
    async fn test_add_step_success() {
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id: trial_id.clone(),
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.steps().len(), 1);
        assert_eq!(updated.steps()[0].name(), "捏ね");

        // リポジトリに保存されていることを確認
        let saved = uow.trial_repository().find_by_id(&trial_id).await.unwrap();
        assert!(saved.is_some());
        assert_eq!(saved.unwrap().steps().len(), 1);
    }

    #[tokio::test]
    async fn test_add_step_with_parameters() {
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![ParameterInput {
                content: ParameterContent::KeyValue {
                    key: "強力粉".to_string(),
                    value: ParameterValue::Quantity {
                        amount: 300.0,
                        unit: "g".to_string(),
                    },
                },
            }],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.steps()[0].parameters().len(), 1);
    }

    #[tokio::test]
    async fn test_returns_error_when_trial_not_found() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: TrialId::new(),
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::TrialNotFound);
    }

    #[tokio::test]
    async fn test_returns_domain_error_when_trial_completed() {
        let trial = completed_trial();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(domain_add_step::Error::TrialAlreadyCompleted)
        );
    }
}

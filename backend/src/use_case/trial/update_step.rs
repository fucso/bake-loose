//! update_step ユースケース
//!
//! 既存の Step の名前・開始日時・パラメーターを更新する。

use chrono::{DateTime, Utc};

use crate::domain::actions::trial::update_step;
use crate::domain::models::parameter::{ParameterContent, ParameterId};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
pub struct Input {
    pub trial_id: TrialId,
    pub step_id: StepId,
    pub name: Option<String>,
    pub started_at: Option<Option<DateTime<Utc>>>,
    pub add_parameters: Vec<ParameterInput>,
    pub remove_parameter_ids: Vec<ParameterId>,
}

/// パラメーター入力
pub struct ParameterInput {
    pub content: ParameterContent,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(update_step::Error),
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
        Some(t) => t,
        None => {
            let _ = uow.rollback().await;
            return Err(Error::TrialNotFound);
        }
    };

    // 3. ドメインアクション用のコマンドを構築
    let command = update_step::Command {
        step_id: input.step_id,
        name: input.name,
        started_at: input.started_at,
        add_parameters: input
            .add_parameters
            .into_iter()
            .map(|p| update_step::ParameterInput { content: p.content })
            .collect(),
        remove_parameter_ids: input.remove_parameter_ids,
    };

    // 4. ドメインアクション実行
    let updated_trial = match update_step::run(trial, command) {
        Ok(t) => t,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 5. 永続化
    if let Err(e) = uow.trial_repository().save(&updated_trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 6. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(updated_trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{ParameterContent, ParameterValue};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::domain::models::trial::TrialStatus;
    use crate::use_case::test::MockUnitOfWork;

    fn make_trial_with_step() -> (Trial, StepId) {
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step = Step::new(trial_id.clone(), "準備".to_string(), 1);
        let step_id = step.id().clone();
        let now = Utc::now();
        let trial = Trial::from_raw(
            trial_id,
            project_id,
            Some("テスト試行".to_string()),
            None,
            TrialStatus::InProgress,
            vec![step],
            now,
            now,
        );
        (trial, step_id)
    }

    fn make_trial_with_step_and_parameter() -> (Trial, StepId, ParameterId) {
        use crate::domain::models::parameter::Parameter;
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step_id = StepId::new();
        let param_id = ParameterId::new();
        let now = Utc::now();
        let param = Parameter::from_raw(
            param_id.clone(),
            step_id.clone(),
            ParameterContent::Text {
                value: "テキスト".to_string(),
            },
            now,
            now,
        );
        let step = Step::from_raw(
            step_id.clone(),
            trial_id.clone(),
            "準備".to_string(),
            1,
            None,
            None,
            vec![param],
            now,
            now,
        );
        let trial = Trial::from_raw(
            trial_id,
            project_id,
            None,
            None,
            TrialStatus::InProgress,
            vec![step],
            now,
            now,
        );
        (trial, step_id, param_id)
    }

    // --- 正常系 ---

    #[tokio::test]
    async fn test_update_step_name() {
        let (trial, step_id) = make_trial_with_step();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id: step_id.clone(),
            name: Some("新しいステップ名".to_string()),
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.name(), "新しいステップ名");
    }

    #[tokio::test]
    async fn test_add_parameters() {
        let (trial, step_id) = make_trial_with_step();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id: step_id.clone(),
            name: None,
            started_at: None,
            add_parameters: vec![ParameterInput {
                content: ParameterContent::KeyValue {
                    key: "粉".to_string(),
                    value: ParameterValue::Text("強力粉".to_string()),
                },
            }],
            remove_parameter_ids: vec![],
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 1);
    }

    #[tokio::test]
    async fn test_remove_parameters() {
        let (trial, step_id, param_id) = make_trial_with_step_and_parameter();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id: step_id.clone(),
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![param_id],
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 0);
    }

    // --- 異常系 ---

    #[tokio::test]
    async fn test_returns_error_when_trial_not_found() {
        let mut uow = MockUnitOfWork::default();
        let non_existent_trial_id = TrialId::new();
        let step_id = StepId::new();

        let input = Input {
            trial_id: non_existent_trial_id,
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::TrialNotFound);
    }

    #[tokio::test]
    async fn test_returns_domain_error_when_step_not_found() {
        let (trial, _step_id) = make_trial_with_step();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);
        let non_existent_step_id = StepId::new();

        let input = Input {
            trial_id,
            step_id: non_existent_step_id,
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(update_step::Error::StepNotFound)
        );
    }
}

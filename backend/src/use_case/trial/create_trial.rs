//! create_trial ユースケース

use crate::domain::actions::trial::create_trial;
use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
pub struct Input {
    pub project_id: ProjectId,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub steps: Vec<create_trial::StepInput>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(create_trial::Error),
    ProjectNotFound,
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. プロジェクトの存在確認
    let project = uow
        .project_repository()
        .find_by_id(&input.project_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    if project.is_none() {
        let _ = uow.rollback().await;
        return Err(Error::ProjectNotFound);
    }

    // 3. ドメインアクション実行
    let command = create_trial::Command {
        project_id: input.project_id,
        name: input.name,
        memo: input.memo,
        steps: input.steps,
    };
    let trial = match create_trial::run(command) {
        Ok(t) => t,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.trial_repository().save(&trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::trial::create_trial::{ParameterInput, StepInput};
    use crate::domain::models::parameter::{DurationUnit, DurationValue, ParameterContent, ParameterValue};
    use crate::domain::models::project::Project;
    use crate::use_case::test::MockUnitOfWork;

    async fn setup_uow_with_project() -> (MockUnitOfWork, ProjectId) {
        let project = Project::new("テストプロジェクト".to_string());
        let project_id = project.id().clone();
        let mut uow = MockUnitOfWork::default();
        // 直接保存してプロジェクトをセットアップ（トランザクション不要）
        uow.project_repository().save(&project).await.unwrap();
        (uow, project_id)
    }

    // --- 正常系 ---

    #[tokio::test]
    async fn test_create_trial_success() {
        let (mut uow, project_id) = setup_uow_with_project().await;

        let input = Input {
            project_id: project_id.clone(),
            name: Some("バゲット第1回".to_string()),
            memo: Some("初めての試行".to_string()),
            steps: vec![],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        assert_eq!(trial.project_id(), &project_id);
        assert_eq!(trial.name(), Some("バゲット第1回"));
        assert_eq!(trial.memo(), Some("初めての試行"));
        assert!(trial.steps().is_empty());

        // リポジトリに保存されていることを確認
        let saved = uow.trial_repository().find_by_id(trial.id()).await.unwrap();
        assert!(saved.is_some());
    }

    #[tokio::test]
    async fn test_create_trial_with_steps() {
        let (mut uow, project_id) = setup_uow_with_project().await;

        let input = Input {
            project_id,
            name: None,
            memo: None,
            steps: vec![
                StepInput {
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
                },
                StepInput {
                    name: "一次発酵".to_string(),
                    started_at: None,
                    parameters: vec![ParameterInput {
                        content: ParameterContent::Duration {
                            duration: DurationValue::new(60.0, DurationUnit::Minute),
                            note: "一次発酵時間".to_string(),
                        },
                    }],
                },
            ],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        assert_eq!(trial.steps().len(), 2);
        assert_eq!(trial.steps()[0].name(), "捏ね");
        assert_eq!(trial.steps()[1].name(), "一次発酵");
    }

    // --- 異常系 ---

    #[tokio::test]
    async fn test_returns_error_when_project_not_found() {
        let mut uow = MockUnitOfWork::default();
        let nonexistent_id = ProjectId::new();

        let input = Input {
            project_id: nonexistent_id,
            name: Some("テスト".to_string()),
            memo: None,
            steps: vec![],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::ProjectNotFound);
    }

    #[tokio::test]
    async fn test_returns_domain_error() {
        let (mut uow, project_id) = setup_uow_with_project().await;

        let input = Input {
            project_id,
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "".to_string(), // 空のステップ名 → ドメインエラー
                started_at: None,
                parameters: vec![],
            }],
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(create_trial::Error::EmptyStepName { step_index: 0 })
        );
    }
}

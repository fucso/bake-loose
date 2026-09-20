//! create_project ユースケース

use crate::domain::actions::project::create_project;
use crate::domain::models::project::Project;
use crate::ports::error::RepositoryError;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::unit_of_work::UnitOfWork;
use crate::use_case::project::save_project;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub name: String,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(create_project::Error),
    DuplicateName,
    /// 一意制約違反など、並行操作との競合（リトライで解消し得る）
    Conflict {
        entity: String,
        field: String,
    },
    Infrastructure(String),
}

impl From<RepositoryError> for Error {
    fn from(error: RepositoryError) -> Self {
        match error {
            // 事前の重複チェックとの競合ウィンドウで DB 側が検出した名前の重複は、
            // ユーザーから見れば同じ「名前の重複」なので DuplicateName に寄せる。
            // 将来 projects に別の一意制約（slug など）が追加されたときに
            // 無関係な競合まで「名前が重複」と表示しないよう、違反フィールドで絞り込む。
            // DB が検出した競合はここでユーザー向けの種別に畳まれ、制約の情報が失われる。
            // 本番で競合が多発したときに追跡できるよう、畳む直前にログへ残す。
            RepositoryError::Conflict {
                ref entity,
                ref field,
            } if field == "name" => {
                log::warn!("Conflict folded into DuplicateName: {}.{}", entity, field);
                Error::DuplicateName
            }
            // 名前以外の一意制約違反も並行操作との競合であり内部エラーではない。
            // 制約名が取得できず field が "unknown" になった場合もここに落ちるため、
            // 内部エラーではなく「競合のためリトライを促す」表示になる。
            RepositoryError::Conflict { entity, field } => Error::Conflict { entity, field },
            other => Error::Infrastructure(format!("{:?}", other)),
        }
    }
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Project, Error> {
    // 1. 重複チェック
    if uow
        .project_repository()
        .exists_by_name(&input.name)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        return Err(Error::DuplicateName);
    }

    // 2. ドメインアクション実行
    let command = create_project::Command { name: input.name };
    let project = create_project::run(command).map_err(Error::Domain)?;

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_project(uow, &project).await?;

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::project::create_project;
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_execute_creates_project_successfully() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            name: "新規プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.name(), "新規プロジェクト");

        // モックのリポジトリに保存されていることを確認
        let saved_project = uow
            .project_repository()
            .find_by_id(project.id())
            .await
            .unwrap();
        assert!(saved_project.is_some());
    }

    #[tokio::test]
    async fn test_execute_returns_duplicate_error_when_name_exists() {
        let mut uow = MockUnitOfWork::default();

        // 既存プロジェクトを作成（トランザクションなしで直接保存）
        let existing_project = Project::new("既存プロジェクト".to_string());
        uow.project_repository()
            .save(&existing_project)
            .await
            .unwrap();

        let input = Input {
            name: "既存プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::DuplicateName);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_empty_name() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            name: "".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(create_project::Error::EmptyName)
        );
    }

    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let mut uow = MockUnitOfWork::default();
        // 永続化だけを失敗させる
        uow.fail_save();
        let input = Input {
            name: "新規プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(matches!(result, Err(Error::Infrastructure(_))));
        // ロールバックが呼ばれ、コミットは呼ばれていないこと
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(
            uow.rollback_success_count(),
            1,
            "ROLLBACK が実際に発行されていない"
        );
        assert_eq!(uow.commit_count(), 0);
    }

    /// 事前チェックとの競合ウィンドウで DB が検出した重複も DuplicateName として返す
    #[tokio::test]
    async fn test_execute_returns_duplicate_name_when_save_violates_unique_constraint() {
        let mut uow = MockUnitOfWork::default();
        uow.fail_save_with(RepositoryError::Conflict {
            entity: "project".to_string(),
            field: "name".to_string(),
        });
        let input = Input {
            name: "新規プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::DuplicateName);
        // ロールバックが呼ばれ、コミットは呼ばれていないこと
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(
            uow.rollback_success_count(),
            1,
            "ROLLBACK が実際に発行されていない"
        );
        assert_eq!(uow.commit_count(), 0);
    }

    /// name 以外の一意制約違反は DuplicateName に畳まず Conflict として扱う
    ///
    /// 将来 projects に別の一意制約が追加されたとき、
    /// 無関係な競合まで「同じ名前のプロジェクトが既に存在します」と表示しないこと。
    ///
    /// 以前は Infrastructure に畳んでいたが、DB が検出した制約違反は
    /// サーバー内部の障害ではなくリトライで解消し得る競合であるため Conflict に改めた。
    #[tokio::test]
    async fn test_execute_does_not_map_non_name_conflict_to_duplicate_name() {
        let mut uow = MockUnitOfWork::default();
        uow.fail_save_with(RepositoryError::Conflict {
            entity: "project".to_string(),
            field: "slug".to_string(),
        });
        let input = Input {
            name: "新規プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Conflict {
                entity: "project".to_string(),
                field: "slug".to_string(),
            }
        );
    }

    /// 制約名が取得できず field が "unknown" になった競合も Conflict として扱う
    ///
    /// 本来は名前の重複かもしれないが、判別できない以上 DuplicateName とは言い切れない。
    /// 内部エラーではなくリトライを促す競合に倒す。
    #[tokio::test]
    async fn test_execute_maps_unknown_field_conflict_to_conflict() {
        let mut uow = MockUnitOfWork::default();
        uow.fail_save_with(RepositoryError::Conflict {
            entity: "project".to_string(),
            field: "unknown".to_string(),
        });
        let input = Input {
            name: "新規プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Conflict {
                entity: "project".to_string(),
                field: "unknown".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_too_long_name() {
        let mut uow = MockUnitOfWork::default();
        let long_name = "a".repeat(101);
        let input = Input { name: long_name };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(create_project::Error::NameTooLong {
                max: 100,
                actual: 101
            })
        );
    }
}

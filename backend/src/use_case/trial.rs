//! Trial ユースケース
//!
//! 試行（Trial）関連のユースケースを集約する。

pub mod add_parameter;
pub mod add_step;
pub mod complete_step;
pub mod complete_trial;
pub mod create_trial;
pub mod get_trial;
pub mod list_trials_by_project;
pub mod remove_parameter;
pub mod update_parameter;
pub mod update_step;
pub mod update_trial;

use crate::domain::models::trial::Trial;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{RepositoryError, UnitOfWork};

/// 開始済みトランザクション内で Trial を保存し、失敗時はロールバックする
///
/// 書き込みユースケースの「4. 永続化」は必ずこのヘルパーを経由すること。
///
/// リポジトリはトランザクションの `Arc` を clone して保持するため、
/// `save()` の結果を判定する前にリポジトリを drop させないと
/// `rollback()` 内の `Arc::try_unwrap` が失敗し、
/// 明示的な ROLLBACK が発行されない（詳細は `.claude/rules/backend/use-case.md`）。
/// ブロックでリポジトリの生存範囲を閉じる形をここに閉じ込め、
/// 呼び出し側が危険な形を書く余地を無くす。
pub(crate) async fn save_trial<U: UnitOfWork>(
    uow: &mut U,
    trial: &Trial,
) -> Result<(), RepositoryError> {
    let save_result = {
        let repo = uow.trial_repository();
        repo.save(trial).await
    };

    if let Err(error) = save_result {
        if let Err(rollback_error) = uow.rollback().await {
            log::error!("rollback failed: {:?}", rollback_error);
        }
        return Err(error);
    }

    Ok(())
}

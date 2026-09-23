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
use crate::ports::trial_repository::{TrialRepository, TrialScope};
use crate::ports::{RepositoryError, UnitOfWork};
use crate::use_case::rollback_on_error;

/// 書き込みユースケースが find に使う scope を決定する
///
/// `write_scope` はそのユースケースが実際に書き込むレイヤー、
/// `return_scope` は呼び出し元（リゾルバー）が戻り値として必要とするレイヤー。
/// 書き込みユースケースは find した集約を加工してそのまま返すため、
/// `write_scope` だけで find すると `return_scope` が要求するレイヤーが
/// 空のまま返り、GraphQL レスポンスから下位レイヤーが黙って消える。
///
/// save は `write_scope` のまま行うため、戻り値のために読み込んだ下位レイヤーが
/// DB へ書き戻されることはない（詳細は `.claude/rules/backend/repository.md`）。
pub(crate) fn read_scope_for_write(
    write_scope: TrialScope,
    return_scope: TrialScope,
) -> TrialScope {
    write_scope.max(return_scope)
}

/// 開始済みトランザクション内で Trial を保存し、失敗時はロールバックする
///
/// 書き込みユースケースの「4. 永続化」は必ずこのヘルパーを経由すること。
/// `scope` はそのユースケースが実際に読み書きするレイヤーの深さに応じて呼び出し側が指定する
/// （詳細は `.claude/rules/backend/repository.md`）。
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
    scope: TrialScope,
) -> Result<(), RepositoryError> {
    let save_result = {
        let repo = uow.trial_repository();
        repo.save(trial, scope).await
    };

    rollback_on_error(uow, save_result).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_scope_for_write_expands_to_return_scope() {
        assert_eq!(
            read_scope_for_write(TrialScope::TrialOnly, TrialScope::Full),
            TrialScope::Full
        );
        assert_eq!(
            read_scope_for_write(TrialScope::WithSteps, TrialScope::Full),
            TrialScope::Full
        );
    }

    #[test]
    fn test_read_scope_for_write_keeps_write_scope_when_return_scope_is_shallower() {
        assert_eq!(
            read_scope_for_write(TrialScope::WithSteps, TrialScope::TrialOnly),
            TrialScope::WithSteps
        );
        assert_eq!(
            read_scope_for_write(TrialScope::TrialOnly, TrialScope::TrialOnly),
            TrialScope::TrialOnly
        );
    }
}

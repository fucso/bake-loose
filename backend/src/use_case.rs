//! UseCase層
//!
//! ドメインアクションを組み合わせてビジネスフローを実現するオーケストレーション層。
//! domain層とports層にのみ依存する。

pub mod project;
pub mod trial;

#[cfg(test)]
pub mod test;

use crate::ports::UnitOfWork;

/// 開始済みトランザクション内の書き込み結果を判定し、失敗時はロールバックする
///
/// 集約ごとの save ヘルパー（`project::save_project` / `trial::save_trial`）から呼ぶ。
/// ロールバック失敗時のログ方針をここ 1 箇所に集約し、集約が増えても方針が分岐しないようにする。
pub(crate) async fn rollback_on_error<U: UnitOfWork, E>(
    uow: &mut U,
    result: Result<(), E>,
) -> Result<(), E> {
    if let Err(error) = result {
        if let Err(rollback_error) = uow.rollback().await {
            log::error!("rollback failed: {:?}", rollback_error);
        }
        return Err(error);
    }

    Ok(())
}

//! get_trial ユースケース
//!
//! IDでトライアルを取得する。

use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

#[derive(Debug)]
pub enum Error {
    Infrastructure(String),
}

/// IDでトライアルを取得する
///
/// 読み取り専用のためトランザクションは不要。
pub async fn execute<U: UnitOfWork>(uow: &mut U, id: &TrialId) -> Result<Option<Trial>, Error> {
    uow.trial_repository()
        .find_by_id(id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

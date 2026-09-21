//! TrialRepository トレイト

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::error::RepositoryError;

/// find/save が処理対象とする Trial 集約のレイヤー深さ
///
/// 段階的な depth の順序を持つ（`TrialOnly` < `WithSteps` < `Full`）。
/// scope 未指定に相当する呼び出しは `Full` を使うことで、既存の全レイヤー
/// 取得・全レイヤー保存の振る舞いと後方互換になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TrialScope {
    /// Trial本体のみ（Step/Parameterは含まない）
    TrialOnly,
    /// Trial + Step（Parameterは含まない）
    WithSteps,
    /// Trial + Step + Parameter（従来の全量取得・全量保存と同等）
    #[default]
    Full,
}

#[async_trait::async_trait]
pub trait TrialRepository: Send + Sync {
    async fn find_by_id(
        &self,
        id: &TrialId,
        scope: TrialScope,
    ) -> Result<Option<Trial>, RepositoryError>;

    async fn find_all_by_project(
        &self,
        project_id: &ProjectId,
        scope: TrialScope,
    ) -> Result<Vec<Trial>, RepositoryError>;

    /// Trialを保存（新規作成または更新）する
    ///
    /// `scope` は洗い替え（差分削除 + upsert）の対象範囲を制御する。
    /// `TrialOnly` は Trial 本体のみ、`WithSteps` は Trial + Step、
    /// `Full` は Trial + Step + Parameter を洗い替える。
    /// scope が満たさないレイヤーには一切アクセスしない
    /// （例: `TrialOnly` では Step の差分削除すら発行しない）ため、
    /// 部分スコープで取得した Trial（下位レイヤーが空）をそのまま渡しても
    /// 既存の Step/Parameter を消失させない。
    async fn save(&self, trial: &Trial, scope: TrialScope) -> Result<(), RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_scope_orders_by_depth() {
        assert!(TrialScope::TrialOnly < TrialScope::WithSteps);
        assert!(TrialScope::WithSteps < TrialScope::Full);
        assert!(TrialScope::TrialOnly < TrialScope::Full);
    }

    #[test]
    fn test_trial_scope_default_is_full() {
        assert_eq!(TrialScope::default(), TrialScope::Full);
    }
}

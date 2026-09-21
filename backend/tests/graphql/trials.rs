//! Trial に関する GraphQL テスト
//!
//! mutation / クエリ単位でファイルを分割している。
//! Trial 自体を対象とするテストは `trial_` プレフィックスを付与する。

pub mod helpers;
pub mod lifecycle;
pub mod parameter_add;
pub mod parameter_remove;
pub mod parameter_update;
pub mod step_add;
pub mod step_complete;
pub mod step_update;
pub mod trial_complete;
pub mod trial_create;
pub mod trial_get;
pub mod trial_list;
pub mod trial_update;

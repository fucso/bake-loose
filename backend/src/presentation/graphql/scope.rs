//! GraphQL の selection set から `TrialScope` を組み立てるヘルパー
//!
//! read（query）だけでなく write（mutation）でも同じ仕組みを使う。
//! mutation はユースケースが返した集約をそのままレスポンスへ載せるため、
//! 戻り値として要求されたレイヤーを scope に含めないと、DB にデータが
//! 存在していても空配列を返してしまう。

use async_graphql::Lookahead;

use crate::ports::trial_repository::TrialScope;

/// `Trial` を返すフィールドの selection set から `TrialScope` を組み立てる
///
/// `steps { parameters { ... } }` まで要求されていれば `Full`、
/// `steps { ... }` のみ（`parameters` 未選択）なら `WithSteps`、
/// `steps` 自体が選択されていなければ `TrialOnly` とする。
pub fn trial_scope_from_look_ahead(look_ahead: Lookahead<'_>) -> TrialScope {
    let steps = look_ahead.field("steps");
    if !steps.exists() {
        return TrialScope::TrialOnly;
    }

    if steps.field("parameters").exists() {
        TrialScope::Full
    } else {
        TrialScope::WithSteps
    }
}

/// `Step` を返すフィールドの selection set から `TrialScope` を組み立てる
///
/// Step を返す時点で Trial 集約の Step レイヤーは必須のため最低でも `WithSteps`、
/// `parameters` まで要求されていれば `Full` とする。
pub fn step_scope_from_look_ahead(look_ahead: Lookahead<'_>) -> TrialScope {
    if look_ahead.field("parameters").exists() {
        TrialScope::Full
    } else {
        TrialScope::WithSteps
    }
}

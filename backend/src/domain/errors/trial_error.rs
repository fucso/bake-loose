//! trial ドメインで発生し得るエラーを一元管理する
//!
//! `validators/trial/*` はこの `Error` を直接返し、`actions/trial/*` は
//! `validator::func()?` の形でそのまま伝播させる。アクション個別の
//! `impl From<validator::Error> for Error` によるマッピングは行わない。
//!
//! バリデーターが発見できないエラー（例: `ParameterNotFound`）も、アクションの
//! `validate()` 内でこの `Error` を直接構築して返す。

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Trial は既に完了している
    TrialAlreadyCompleted,
    /// 指定された Step が見つからない
    StepNotFound,
    /// Step は既に完了している
    StepAlreadyCompleted,
    /// 指定された Parameter が見つからない
    ParameterNotFound,
    /// Parameter の種類（バリアント）は変更できない
    ParameterContentTypeMismatch,
    /// Step 名が空
    EmptyStepName,
    /// Step 名が上限文字数を超えている
    StepNameTooLong { max: usize, actual: usize },
    /// Trial 名が空
    EmptyTrialName,
    /// Trial 名が上限文字数を超えている
    TrialNameTooLong { max: usize, actual: usize },
    /// Duration/TimeMarker の値が負
    NegativeDurationValue,
    /// KeyValue の Quantity の unit が空
    EmptyQuantityUnit,
    /// KeyValue の Quantity の amount が0以下
    NonPositiveQuantityAmount,
}

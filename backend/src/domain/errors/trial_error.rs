#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
    ParameterNotFound,
    ParameterContentTypeMismatch,
    EmptyStepName,
    StepNameTooLong { max: usize, actual: usize },
    EmptyTrialName,
    TrialNameTooLong { max: usize, actual: usize },
    NegativeDurationValue,
    EmptyQuantityUnit,
    NonPositiveQuantityAmount,
}

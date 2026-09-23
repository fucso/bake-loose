#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyName,
    NameTooLong { max: usize, actual: usize },
}

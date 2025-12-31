//! UseCase層
//!
//! ドメインアクションを組み合わせてビジネスフローを実現するオーケストレーション層。
//! domain層とports層にのみ依存する。

pub mod project;

#[cfg(test)]
pub mod test;

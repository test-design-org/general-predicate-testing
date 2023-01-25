use thiserror::Error;

#[derive(Error, Debug)]
pub enum GPTError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unknown Parse error: {0}")]
    UnknownParseError(String),

    #[error("Interval error in logic: {0}")]
    IntervalError(String),

    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, GPTError>;

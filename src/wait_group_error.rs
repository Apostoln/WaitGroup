use std::fmt;

pub enum WaitGroupError {
    NegativeCounter(isize),
    Unexpected(String),
}

impl fmt::Debug for WaitGroupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WaitGroupError::NegativeCounter(counter) => {
                write!(f, "Counter is negative: {}", counter)
            }
            WaitGroupError::Unexpected(description) => {
                write!(f, "Unexpected WaitGroupError: {}", description)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, WaitGroupError>;

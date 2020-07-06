use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ListError {
    pub message: String,
}

impl Error for ListError {}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

use std::fmt;

#[derive(Debug)]
pub enum CustomError {
    InvalidTable { message: String },
    InvalidColumn { message: String },
    InvalidSyntax { message: String },
    GenericError { message: String },
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::InvalidTable { message } => write!(f, "INVALID_TABLE: {}", message),
            CustomError::InvalidColumn { message } => write!(f, "INVALID_COLUMN: {}", message),
            CustomError::InvalidSyntax { message } => write!(f, "INVALID_SYNTAX: {}", message),
            CustomError::GenericError { message } => write!(f, "ERROR: {}", message),
        }
    }
}

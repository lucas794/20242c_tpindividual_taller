use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum TPErrors<'a> {
    InvalidTable(&'a str),
    InvalidSyntax(&'a str),
    InvalidGeneric(&'a str),
}

impl<'a> Display for TPErrors<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TPErrors::InvalidGeneric(e) => write!(f, "[ERROR] {}", *e),
            TPErrors::InvalidTable(e) => write!(f, "[INVALID_TABLE]: Error {}", *e),
            TPErrors::InvalidSyntax(e) => write!(f, "[INVALID_SYNTAX]: Error {}", *e),
        }
    }
}

// implementing the error trait for TPErrors
impl<'a> std::error::Error for TPErrors<'a> {}

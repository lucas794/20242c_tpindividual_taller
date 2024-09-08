use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum TPErrors<'a> {
    Table(&'a str),
    Syntax(&'a str),
    Generic(&'a str),
}

impl<'a> Display for TPErrors<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TPErrors::Generic(e) => write!(f, "[ERROR] {}", *e),
            TPErrors::Table(e) => write!(f, "[INVALID_TABLE]: {}", *e),
            TPErrors::Syntax(e) => write!(f, "[INVALID_SYNTAX]: {}", *e),
        }
    }
}

// implementing the error trait for TPErrors
impl<'a> std::error::Error for TPErrors<'a> {}

use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Tperrors<'a> {
    Table(&'a str),
    Syntax(&'a str),
    Generic(&'a str),
}

impl<'a> Display for Tperrors<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Tperrors::Generic(e) => write!(f, "ERROR {}", e),
            Tperrors::Table(e) => write!(f, "INVALID_TABLE: {}", e),
            Tperrors::Syntax(e) => write!(f, "INVALID_SYNTAX: {}", e),
        }
    }
}

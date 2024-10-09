use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Tperrors {
    Table(String),
    Syntax(String),
    Generic(String),
    Column(String),
}

impl Display for Tperrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Tperrors::Generic(e) => write!(f, "ERROR: {}", e),
            Tperrors::Column(e) => write!(f, "INVALID_COLUMN: {}", e),
            Tperrors::Table(e) => write!(f, "INVALID_TABLE: {}", e),
            Tperrors::Syntax(e) => write!(f, "SYNTAX_ERROR: {}", e),
        }
    }
}

use crate::errors::fileerrors::*;
use crate::{errors::tperrors::Tperrors, table::Table};

pub struct Update;

impl Default for Update {
    fn default() -> Self {
        Update::new()
    }
}

impl Update {
    pub fn new() -> Update {
        Update
    }

    /// A valid UPDATE query contains UPDATE and SET AND ends with ;
    ///
    /// if the query is valid, it will return true
    ///
    /// UPDATE table_name SET column1 = value1, column2 = value2 WHERE condition;
    ///
    /// UPDATE table_name SET column1 = value1, column2 = value2;
    ///
    pub fn is_valid_query(&self, query: &str) -> bool {
        let query = query.trim();

        if query.starts_with("UPDATE") && query.contains("SET") {
            match query.chars().last() {
                Some(';') => return true,
                _ => return false,
            }
        }
        false
    }

    pub fn execute_update(
        &self,
        table: &mut Table,
        columns: Vec<String>,
        values: Vec<String>,
        conditions: Option<&str>,
    ) -> Result<(), Tperrors> {
        let resolve = table.resolve_update(columns, values, conditions);

        match resolve {
            Ok(_) => {
                match table.replace_original_with_tempfile() {
                    Ok(_) => {}
                    Err(e) => match e {
                        FileErrors::DeletionFailed => {
                            return Err(Tperrors::Generic("Deletion failed"));
                        }
                        FileErrors::InvalidFile => {
                            return Err(Tperrors::Generic("Error while updating the file"));
                        }
                    },
                }

                Ok(())
            }
            Err(_) => {
                return Err(Tperrors::Syntax("Invalid columns inside the query"));
            }
        }
    }
}

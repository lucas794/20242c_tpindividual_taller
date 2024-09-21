use crate::errors::fileerrors::*;
use crate::errors::tperrors::Tperrors;
use crate::handler_tables::table::*;

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

    /// Execute the update query
    ///
    /// UPDATE table_name SET column1 = value1, column2 = value2 WHERE condition;
    ///
    /// UPDATE table_name SET column1 = value1, column2 = value2;
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
                            return Err(Tperrors::Generic("Deletion failed".to_string()));
                        }
                        FileErrors::InvalidFile => {
                            return Err(Tperrors::Generic(
                                "Error while updating the file".to_string(),
                            ));
                        }
                    },
                }

                Ok(())
            }
            Err(e) => {
                let formatted_error = format!("{}", e);
                let dots = formatted_error.find(":").unwrap_or_default();
                if formatted_error.contains("SYNTAX") {
                    let formatted_error = formatted_error[dots + 1..].trim().to_string();
                    Err(Tperrors::Syntax(formatted_error))
                } else if formatted_error.contains("COLUMN") {
                    let formatted_error = formatted_error[dots + 1..].trim().to_string();
                    Err(Tperrors::Table(formatted_error))
                } else {
                    let formatted_error = formatted_error[dots + 1..].trim().to_string();
                    Err(Tperrors::Generic(formatted_error))
                }
            }
        }
    }
}

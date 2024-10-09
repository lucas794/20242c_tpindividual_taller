use std::io::{BufReader, Cursor, Read, Seek};

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
    pub fn execute_update<R: Read + Seek>(
        &self,
        table: &mut Table<R>,
        columns: Vec<String>,
        values: Vec<String>,
        conditions: Option<&str>,
    ) -> Result<(), Tperrors> {
        let resolve = table.resolve_update_for_file(columns, values, conditions);

        match resolve {
            Ok(temporal_directory_filename) => {
                match table.replace_original_with(temporal_directory_filename) {
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
                    let formatted_error = formatted_error[dots..].trim().to_string();
                    Err(Tperrors::Syntax(formatted_error))
                } else if formatted_error.contains("COLUMN") {
                    let formatted_error = formatted_error[dots..].trim().to_string();
                    Err(Tperrors::Table(formatted_error))
                } else {
                    let formatted_error = formatted_error[dots..].trim().to_string();
                    Err(Tperrors::Generic(formatted_error))
                }
            }
        }
    }

    /// Function that will execute the update query for the mock table
    ///
    /// Uses the same arguments as the normal execute_update function
    ///
    /// The difference is that this function will return a `BufReader<Cursor<Vec<u8>>>``
    ///
    /// This is because the mock table is not a file, so we need to return the data in a different way
    pub fn execute_update_mock<R: Read + Seek>(
        &self,
        table: &mut Table<R>,
        columns: Vec<String>,
        values: Vec<String>,
        conditions: Option<&str>,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, Tperrors> {
        let resolve = table.resolve_update_mock(columns, values, conditions);

        match resolve {
            Ok(buf) => Ok(buf),
            Err(e) => {
                let formatted_error = format!("{}", e);
                let dots = formatted_error.find(":").unwrap_or_default();
                if formatted_error.contains("SYNTAX") {
                    let formatted_error = formatted_error[dots..].trim().to_string();
                    Err(Tperrors::Syntax(formatted_error))
                } else if formatted_error.contains("COLUMN") {
                    let formatted_error = formatted_error[dots..].trim().to_string();
                    Err(Tperrors::Table(formatted_error))
                } else {
                    let formatted_error = formatted_error[dots..].trim().to_string();
                    Err(Tperrors::Generic(formatted_error))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_update_querys_passes() {
        let update = Update::new();

        let query = "UPDATE table_name SET column1 = value1, column2 = value2 WHERE condition;";
        assert_eq!(update.is_valid_query(query), true);

        let query = "UPDATE table_name SET column1 = value1, column2 = value2;";
        assert_eq!(update.is_valid_query(query), true);

        let query = "UPDATE table_name SET column1 = value1, column2 = value2";
        assert_eq!(update.is_valid_query(query), false);

        let query = "UPDATE table_name SET column1 = value1, column2 = value2 WHERE condition";
        assert_eq!(update.is_valid_query(query), false);

        let query = "UPDATE table_name SET column1 = value1, column2 = value2 WHERE condition";
        assert_eq!(update.is_valid_query(query), false);

        let query = "UPDATE table_name SET column1 = value1, column2 = value2 WHERE condition";
        assert_eq!(update.is_valid_query(query), false);
    }
}

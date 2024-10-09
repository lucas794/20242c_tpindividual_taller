use std::io::{BufReader, Cursor, Read, Seek};

use crate::errors::{fileerrors::FileErrors, tperrors::Tperrors};
use crate::handler_tables::table::Table;

/// Struct to handle the DELETE query.
pub struct Delete;

impl Default for Delete {
    fn default() -> Self {
        Delete::new()
    }
}

impl Delete {
    pub fn new() -> Delete {
        Delete
    }
    /// A valid DELETE query contains DELETE and FROM AND ends with ;
    pub fn is_valid_query(&self, query: &str) -> bool {
        let query = query.trim();

        if query.starts_with("DELETE") && query.contains("FROM") {
            match query.chars().last() {
                Some(';') => return true,
                _ => return false,
            }
        }
        false
    }
    /// Execute the delete query
    pub fn execute_delete<R: Read + Seek>(
        &self,
        table: &mut Table<R>,
        conditions: Option<&str>,
    ) -> Result<(), Tperrors> {
        let resolve = table.resolve_delete_for_file(conditions);
        match resolve {
            Ok(temp_file_dir) => {
                match table.replace_original_with(temp_file_dir) {
                    Ok(_) => {
                        Ok(()) // everything done propertly.
                    }
                    Err(e) => match e {
                        FileErrors::DeletionFailed => {
                            Err(Tperrors::Generic("Deletion failed".to_string()))
                        }
                        FileErrors::InvalidFile => Err(Tperrors::Generic(
                            "Error while updating the file".to_string(),
                        )),
                    },
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Execute the delete query
    ///
    /// This function is used for testing purposes only.
    ///
    /// It will return a `BufReader<Cursor<Vec<u8>>>` with the content of the file.
    pub fn execute_delete_mock<R: Read + Seek>(
        &self,
        table: &mut Table<R>,
        conditions: Option<&str>,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, Tperrors> {
        let resolve = table.resolve_delete_mock(conditions);

        match resolve {
            Ok(b) => Ok(b),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_query() {
        let delete = Delete;
        let query = "DELETE FROM table;";
        assert_eq!(delete.is_valid_query(query), true);

        let query = "DELETE FROM table";
        assert_eq!(delete.is_valid_query(query), false);
    }
}

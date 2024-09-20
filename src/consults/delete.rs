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
    pub fn execute_delete(
        &self,
        table: &mut Table,
        conditions: Option<&str>,
    ) -> Result<(), Tperrors> {
        let resolve = table.resolve_delete(conditions);

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
            Err(_) => Err(Tperrors::Syntax(
                "Invalid columns inside the query".to_string(),
            )),
        }
    }
}

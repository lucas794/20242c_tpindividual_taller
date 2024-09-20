use crate::{
    errors::{fileerrors::FileErrors, tperrors::Tperrors},
    table::Table,
};

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

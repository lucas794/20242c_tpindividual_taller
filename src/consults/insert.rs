use std::io::{Read, Seek};

use crate::errors::tperrors::Tperrors;
use crate::handler_tables::table::*;

/// Struct to handle the INSERT query.
pub struct Insert;

impl Default for Insert {
    fn default() -> Self {
        Insert::new()
    }
}

impl Insert {
    pub fn new() -> Insert {
        Insert
    }

    /// A valid INSERT query contains INSERT INTO and VALUES AND ends with ;
    ///
    /// if the query is valid, it will return true
    pub fn is_valid_query(&self, query: &str) -> bool {
        let query = query.trim();

        if query.starts_with("INSERT INTO") && query.contains("VALUES") {
            match query.chars().last() {
                Some(';') => return true,
                _ => return false,
            }
        }
        false
    }

    /// Execute the insert query
    pub fn execute_insert<R: Read + Seek>(
        &self,
        table: &mut Table<R>,
        columns: Vec<String>,
        values: Vec<Vec<String>>,
    ) -> Result<(), Tperrors> {
        let resolve = table.resolve_insert(columns, values);

        match resolve {
            Ok(lines) => {
                for line in lines {
                    let line = line.join(",");
                    match table.insert_line_to_csv(line) {
                        Ok(_) => {}
                        Err(_) => {
                            return Err(Tperrors::Generic("Error while inserting line".to_string()))
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Execute the insert query and return the inserted values but using a Mock
    ///
    /// This function is used for testing purposes
    ///
    /// # Arguments
    ///
    /// * `table` - Table where the values will be inserted
    ///
    /// * `columns` - Columns to insert
    ///
    /// * `values` - Values to insert
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Vec<String>>, Tperrors>` - A vector of the inserted values
    pub fn execute_insert_mock<R: Read + Seek>(
        &self,
        table: &mut Table<R>,
        columns: Vec<String>,
        values: Vec<Vec<String>>,
    ) -> Result<Vec<Vec<String>>, Tperrors> {
        table.resolve_insert(columns, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_query() {
        let insert = Insert;
        let query = "INSERT INTO table VALUES ('Juan', 20);";
        assert_eq!(insert.is_valid_query(query), true);

        let query = "INSERT INTO table VALUES ('Juan', 20)";
        assert_eq!(insert.is_valid_query(query), false);

        let query = "INSERT INTO table ('Juan', 20);";
        assert_eq!(insert.is_valid_query(query), false);

        let query = "INSERT INTO table VALUES ('Juan', 20)";
        assert_eq!(insert.is_valid_query(query), false);
    }
}

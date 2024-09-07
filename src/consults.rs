use std::io::{self, Write};

use crate::{errors::TPErrors, table::Table};

/// Select representation for the SQL query
pub struct Select;
pub struct Insert;
pub struct Update;
pub struct Delete;

/// implementation of the select query
/// select uses query general validator
impl Select {
    pub fn new() -> Select {
        Select
    }

    /// A valid select query contains SELECT and FROM AND ends with ;
    /// if the query is valid, it will return true
    pub fn is_valid_query<'a>(&self, query: &'a str) -> bool {
        let query = query.trim();

        if query.starts_with("SELECT") && query.contains("FROM") {
            match query.chars().last() {
                Some(';') => return true,
                _ => return false,
            }
        }
        false
    }

    /// Given a table, columns, conditions and sorting method
    /// executes a SELECT query statement.
    /// Returns ok if the query was executed successfully
    pub fn execute_select(
        &self,
        table: &mut Table,
        columns: Vec<String>,
        conditions: Option<&str>,
        sorting_method: Option<Vec<(String, bool)>>,
    ) -> Result<(), TPErrors> {
        let csv_data = table.resolve_select(columns, conditions, sorting_method);

        match csv_data {
            Ok(data) => {
                // lets write stdout
                let stdout = io::stdout();

                let mut handle = io::BufWriter::new(stdout.lock());

                for line in data {
                    let mut temp_line = line.join(",");
                    temp_line.push_str("\n");
                    let _ = handle.write(temp_line.as_bytes());
                }
                Ok(())
            }
            Err(_) => {
                return Err(TPErrors::InvalidSyntax("Invalid columns inside the query"));
            }
        }
    }
}

impl Insert {
    pub fn new() -> Insert {
        Insert
    }

    /// A valid INSERT query contains INSERT INTO and VALUES AND ends with ;
    /// if the query is valid, it will return true
    pub fn is_valid_query<'a>(&self, query: &'a str) -> bool {
        let query = query.trim();

        if query.starts_with("INSERT INTO") && query.contains("VALUES") {
            match query.chars().last() {
                Some(';') => return true,
                _ => return false,
            }
        }
        false
    }

    pub fn execute_insert(
        &self,
        table: &mut Table,
        columns: Vec<String>,
        values: Vec<String>,
    ) -> Result<(), TPErrors> {
        // we need to check if the columns are valid
        let resolve = table.resolve_insert(columns, values);

        match resolve {
            Ok(line) => {
                let mut line = line.join(",");
                line.push_str("\n");
                match table.insert_line_to_csv(line) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(TPErrors::InvalidGeneric("Error while inserting line"));
                    }
                }
            }
            Err(_) => {
                return Err(TPErrors::InvalidGeneric(
                    "Invalid columns inside the query / mismatch with the table",
                ));
            }
        }
    }
}

impl Update {
    pub fn new() -> Update {
        Update
    }
}

impl Delete {
    pub fn new() -> Delete {
        Delete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_invalid_query() {
        let select = Select::new();
        let invalid_consults: Vec<&str> = Vec::from([
            "name, age FROM table",    // missing select
            "SELECT name, age table;", // missing a coma
            "SELECT name, age",        // missing FROM
        ]);
        for invalid_query in invalid_consults {
            assert_eq!(select.is_valid_query(invalid_query), false);
        }
    }

    #[test]
    fn execute_select_fails_with_invalid_columns() {
        let mut table = Table::new("./test.csv").unwrap();
        let select = Select::new();
        // i'm trying to select a column that does not exist
        let columns = vec!["Trabajo Profesional".to_string()];
        let conditions = None;
        let sorting = None;

        let result = select.execute_select(&mut table, columns, conditions, sorting);

        assert_eq!(result.is_err(), true);
    }
}

use crate::{
    errors::{FileErrors, TPErrors},
    table::Table,
};

/// Select representation for the SQL query
pub struct Select;
pub struct Insert;
pub struct Update;
pub struct Delete;

/// implementation of the select query
/// select uses query general validator
///
impl Default for Select {
    fn default() -> Self {
        Select::new()
    }
}
impl Default for Insert {
    fn default() -> Self {
        Insert::new()
    }
}
impl Default for Update {
    fn default() -> Self {
        Update::new()
    }
}
impl Default for Delete {
    fn default() -> Self {
        Delete::new()
    }
}

impl Select {
    pub fn new() -> Select {
        Select
    }

    /// A valid select query contains SELECT and FROM AND ends with ;
    /// 
    /// if the query is valid, it will return true
    pub fn is_valid_query(&self, query: &str) -> bool {
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
    /// 
    /// executes a SELECT query statement.
    /// 
    /// Returns ok if the query was executed successfully
    /// 
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
                for line in data {
                    let temp_line = line.join(",");
                    println!("{}", temp_line);
                }
                Ok(())
            }
            Err(_) => {
                return Err(TPErrors::Syntax("Invalid columns inside the query"));
            }
        }
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
                line.push('\n');
                match table.insert_line_to_csv(line) {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        return Err(TPErrors::Generic("Error while inserting line"));
                    }
                }
            }
            Err(_) => {
                return Err(TPErrors::Generic(
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
    ) -> Result<(), TPErrors> {
        let resolve = table.resolve_update(columns, values, conditions);

        match resolve {
            Ok(_) => {
                match table.replace_original_with_tempfile() {
                    Ok(_) => {}
                    Err(e) => match e {
                        FileErrors::DeletionFailed => {
                            return Err(TPErrors::Generic("Deletion failed"));
                        }
                        FileErrors::InvalidFile => {
                            return Err(TPErrors::Generic("Error while updating the file"));
                        }
                    },
                }

                Ok(())
            }
            Err(_) => {
                return Err(TPErrors::Syntax("Invalid columns inside the query"));
            }
        }
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
    ) -> Result<(), TPErrors> {
        let resolve = table.resolve_delete(conditions);

        match resolve {
            Ok(_) => {
                match table.replace_original_with_tempfile() {
                    Ok(_) => {}
                    Err(e) => match e {
                        FileErrors::DeletionFailed => {
                            return Err(TPErrors::Generic("Deletion failed"));
                        }
                        FileErrors::InvalidFile => {
                            return Err(TPErrors::Generic("Error while updating the file"));
                        }
                    },
                }

                Ok(())
            }
            Err(_) => {
                return Err(TPErrors::Syntax("Invalid columns inside the query"));
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_invalid_query_throws_error() {
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
        let mut table = Table::new("./tests/database.csv").unwrap();
        let select = Select::new();
        // i'm trying to select a column that does not exist
        let columns = vec!["Trabajo Profesional".to_string()];
        let conditions = None;
        let sorting = None;

        let result = select.execute_select(&mut table, columns, conditions, sorting);

        assert_eq!(result.is_err(), true);
    }
}

use crate::errors::tperrors::*;
use crate::table::Table;

pub struct Select;

impl Default for Select {
    fn default() -> Self {
        Select::new()
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
    ) -> Result<(), Tperrors> {
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
                return Err(Tperrors::Syntax("Invalid columns inside the query"));
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

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
    pub fn execute_insert(
        &self,
        table: &mut Table,
        columns: Vec<String>,
        values: Vec<String>,
    ) -> Result<(), Tperrors> {
        // we need to check if the columns are valid
        let resolve = table.resolve_insert(columns, values);

        match resolve {
            Ok(line) => {
                let mut line = line.join(",");
                line.push('\n');
                match table.insert_line_to_csv(line) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(Tperrors::Generic("Error while inserting line".to_string())),
                }
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

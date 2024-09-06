use core::hash;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, Seek},
};

use crate::{
    conditions::{Conditions, Value},
    errors::TPErrors,
};

pub struct Table<'a> {
    file_name: &'a str,
    file: File,
}

impl<'a> Table<'a> {
    pub fn new(file_name: &'a str) -> Result<Self, std::io::Error> {
        let file_reference = File::open(file_name);

        match file_reference {
            Ok(file) => Ok(Table { file, file_name }),
            Err(e) => {
                println!("[INVALID_TABLE]: Error {}", e);
                // lets throw error and stop the program
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error opening file",
                ))
            }
        }
    }
    /// Get the file name of the table
    /// # Example
    /// ./path/table.csv -> table
    /// ./table.csv -> table
    pub fn get_file_name(&self) -> Result<&'a str, TPErrors<'static>> {
        let table_file = match self.file_name.split("/").last() {
            Some(name) => name,
            None => {
                return Err(TPErrors::InvalidGeneric(
                    "Error getting table file name by unknown reason",
                ));
            }
        };

        // at this point i have table.csv, lets split again by . and get the first element
        let table_name = match table_file.split(".").next() {
            Some(name) => name,
            None => {
                return Err(TPErrors::InvalidGeneric(
                    "Error getting table name, splitting by '.' failed",
                ));
            }
        };
        Ok(table_name)
    }
    /// given the columns of the table, and the conditions of the query (as String)
    /// it will return the result of the query
    pub fn execute_select(
        &mut self,
        columns: Vec<String>,
        opt_conditions_as_str: Option<&str>,
        vector_sorting: Option<Vec<(String, bool)>>,
    ) -> Result<Vec<Vec<String>>, std::io::Error> {
        // we need to match the index of the columns with the index of the csv
        // we need to read the csv and get the columns
        // we need to print the columns

        // lets read the first line of the file
        let index_columns = std::io::BufReader::new(&self.file)
            .lines()
            .next()
            .unwrap_or(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error reading file",
            )))?;

        let splitted_columns = index_columns.split(",").collect::<Vec<&str>>();

        // lets check if its a select *
        let index_columns = if columns.len() == 1 && columns[0] == "*" {
            (0..splitted_columns.len()).collect::<Vec<usize>>()
        } else {
            let temp_index = splitted_columns
                .iter()
                .enumerate()
                .filter(|(_i, c)| columns.contains(&c.to_string()))
                .map(|(i, _c)| i)
                .collect::<Vec<usize>>();

            if columns.len() != temp_index.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid columns",
                ));
            }
            temp_index
        };

        let columns = if columns.len() == 1 && columns[0] == "*" {
            // we need to handle the case if its a joker *
            splitted_columns
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        } else {
            columns
        };

        // and print the columns
        self.file.seek(std::io::SeekFrom::Start(0))?; // lets place it after the columns name
        let mut result: Vec<Vec<String>> = Vec::new();

        for line_read in std::io::BufReader::new(&self.file).lines().skip(1) {
            let line = line_read?;
            let splitted_line = line.split(",").map(|s| s).collect::<Vec<&str>>();

            if opt_conditions_as_str.is_some() {
                // we have conditions to check
                let (extracted_conditions, line_to_writte) =
                    self.extract_conditions(&index_columns, &splitted_line, &columns);

                // now everything is clear and ready to check if conditions are met
                let condition = Conditions::new(extracted_conditions);
                let str_conditions = opt_conditions_as_str.unwrap_or_else(|| "");

                if condition.matches_condition(str_conditions) {
                    result.push(line_to_writte);
                }
            } else {
                // we need to push the matched columns to the vector
                let line_to_writte = index_columns
                    .iter()
                    .map(|i| splitted_line[*i])
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                result.push(line_to_writte);
            }
        }
        result.insert(0, columns); // to prevent clone, at the end the columns at the top of the vector.

        Ok(result)
    }

    /// Given a index of columns, the columns and the splitted line
    /// we return a hash of conditions AND the line itself.
    fn extract_conditions(
        &self,
        index_columns: &Vec<usize>,
        splitted_line: &Vec<&str>,
        columns_from_query: &Vec<String>,
    ) -> (HashMap<String, Value>, Vec<String>) {
        let selected_columns = index_columns
            .iter()
            .map(|i| splitted_line[*i])
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // for each column, we need to map the condition
        let mut hash_conditions: HashMap<String, Value> = HashMap::new();
        for (j, _col) in selected_columns.iter().enumerate() {

            // with this we get the column that we want to check
            let column_condition = columns_from_query[j].as_str();
            // with this we get the value of the column
            let trimmed_value = selected_columns[j].trim().to_string();

            if let Some(v) = trimmed_value.parse::<i64>().ok() {
                hash_conditions.insert(column_condition.to_string(), Value::Integer(v));
            } else {
                hash_conditions.insert(column_condition.to_string(), Value::String(trimmed_value));
            }
        }
        (hash_conditions, selected_columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let table = Table::new("./test.csv").unwrap();
        let filename = table.get_file_name().unwrap();
        assert_eq!(filename, "test");
    }

    #[test]
    fn invalid_table() {
        let invalid_routes = vec!["./invalidtable.csv", "./invalidtable"];

        for invalid_route in invalid_routes {
            let table = Table::new(invalid_route);
            assert_eq!(table.is_err(), true);
        }
    }

    #[test]
    fn invalid_column() {
        let mut table = Table::new("./test.csv").unwrap();

        // tesis is the invalid columns
        let columns = vec!["Edad".to_string(), "Tesis".to_string()];
        let conditions = Some("WHERE name = 'John'");
        let result = table.execute_select(columns, conditions, None);
        assert_eq!(result.is_err(), true);
    }
}

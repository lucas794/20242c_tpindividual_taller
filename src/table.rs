use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
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
            Err(_) => {
                // lets throw error and stop the program
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error opening file",
                ))
            }
        }

        // lets close the file
    }

    pub fn get_file_directory(&self) -> &'a str {
        self.file_name
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
    pub fn resolve_select(
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
                let line_to_write = index_columns
                    .iter()
                    .map(|i| splitted_line[*i])
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                result.push(line_to_write);
            }
        }

        // lets sort the vector if we have a sorting method..
        if let Some(sorting) = vector_sorting {
            // first, let check if the columns are valid
            let columns_from_query = sorting.iter().map(|(c, _)| c).collect::<Vec<&String>>();
            if !columns_from_query.iter().all(|c| columns.contains(c)) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid columns inside the query",
                ));
            }

            // now sorting the vector
            result.sort_by(|a, b| {
                for (column, asc) in &sorting {
                    let index = columns.iter().position(|c| c == column).unwrap();
                    let a_value = a[index].as_str();
                    let b_value = b[index].as_str();

                    let result = if a_value < b_value {
                        Ordering::Less
                    } else if a_value > b_value {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    };

                    if *asc {
                        return result;
                    } else {
                        return result.reverse();
                    }
                }
                Ordering::Equal
            });
        }
        result.insert(0, columns); // to prevent clone, at the end the columns at the top of the vector.

        Ok(result)
    }

    /// given a columns as Vec<String> and values as Vec<String>
    /// It returns the proper line to write in the csv
    /// else returns a Error.
    pub fn resolve_insert(
        &self,
        columns: Vec<String>,
        values: Vec<String>,
    ) -> Result<Vec<String>, std::io::Error> {
        // we need to check if the columns are valid
        let index_columns = std::io::BufReader::new(&self.file)
            .lines()
            .next()
            .unwrap_or(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error reading file",
            )))?;

        let splitted_columns = index_columns.split(",").collect::<Vec<&str>>();

        let temp_index = splitted_columns
            .iter()
            .enumerate()
            .filter(|(_i, c)| columns.contains(&c.to_string()))
            .map(|(i, _c)| i)
            .collect::<Vec<usize>>();

        if columns.len() != temp_index.len()
            || columns
                .iter()
                .any(|c| !splitted_columns.contains(&c.as_str()))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid columns",
            ));
        }

        // now we need to each temp_index, writ the value
        // else we write a empty string
        let mut line_to_write: Vec<String> = Vec::new();
        for (i, _col) in splitted_columns.iter().enumerate() {
            if temp_index.contains(&i) {
                let position = match temp_index.iter().position(|&x| x == i) {
                    Some(p) => p,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Position not found",
                        ));
                    }
                };
                line_to_write.push(values[position].to_string());
            } else {
                line_to_write.push("".to_string());
            }
        }

        Ok(line_to_write)
    }

    /// Function that handles the resolve of the update query
    /// Given the columns to update, the values to update, and the conditions as str
    /// it will return the result of the query
    pub fn resolve_update(
        &mut self,
        columns: Vec<String>,
        values: Vec<String>,
        opt_conditions: Option<&str>,
    ) -> Result<(), std::io::Error> {
        // we need to check if the columns are valid
        let columns_from_csv = std::io::BufReader::new(&self.file)
            .lines()
            .next()
            .unwrap_or(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error reading file",
            )))?;

        let splitted_columns = columns_from_csv.split(",").collect::<Vec<&str>>();

        let index_selected_column = splitted_columns
            .iter()
            .enumerate()
            .filter(|(_i, c)| columns.contains(&c.to_string()))
            .map(|(i, _c)| i)
            .collect::<Vec<usize>>();

        let index_all_columns = (0..splitted_columns.len()).collect::<Vec<usize>>();

        // we need to change the value of the columns
        // we use a hash to store the new values
        // and keys the index of the columns of change
        // the change is done if the conditions are met
        let mut hash_changes: HashMap<usize, String> = HashMap::new();

        for (i, _col) in splitted_columns.iter().enumerate() {
            if index_selected_column.contains(&i) {
                let position = match index_selected_column.iter().position(|&x| x == i) {
                    Some(p) => p,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Position not found",
                        ));
                    }
                };
                hash_changes.insert(i, values[position].to_string());
            }
        }

        // now we need to read the line and check if condition is met
        // if it is met, we need to change the values
        // and push it to the result

        self.file.seek(SeekFrom::Start(0))?;

        let mut temporal_file = BufWriter::new(File::create("./temp_file.csv")?);
        temporal_file.write_all(
            columns_from_csv
                .split(",")
                .collect::<Vec<&str>>()
                .join(",")
                .as_bytes(),
        )?;
        temporal_file.write_all("\n".as_bytes())?;

        for line in BufReader::new(&self.file).lines().into_iter().skip(1) {
            let line = line?;
            let splitted_line = line.split(",").map(|s| s).collect::<Vec<&str>>();

            match opt_conditions {
                Some(str_conditions) => {
                    let splitted_columns_as_string = splitted_columns
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();
                    let (hashed_conditions, _) = self.extract_conditions(
                        &index_all_columns,
                        &splitted_line,
                        &splitted_columns_as_string,
                    );

                    // lets print the conditions

                    // lets see all keys and values
                    let condition = Conditions::new(hashed_conditions);

                    // lets see all keys and values
                    if condition.matches_condition(str_conditions) {
                        // criteria reached, we need to change the index
                        // of the columns according to the hash database with the proper value
                        let mut new_line = splitted_line.to_vec();
                        for (i, value) in hash_changes.iter() {
                            new_line[*i] = value;
                        }
                        // convert it as Vec<String

                        temporal_file.write_all(new_line.join(",").as_bytes())?;
                        temporal_file.write_all("\n".as_bytes())?;
                    } else {
                        temporal_file.write_all(splitted_line.join(",").as_bytes())?;
                        temporal_file.write_all("\n".as_bytes())?;
                    }
                }
                None => {
                    // we need to change the values
                    let mut new_line = splitted_line.to_vec();
                    for (i, value) in hash_changes.iter() {
                        new_line[*i] = value;
                    }
                    temporal_file.write_all(new_line.join(",").as_bytes())?;
                    temporal_file.write_all("\n".as_bytes())?;
                }
            }
        }
        Ok(())
    }

    /// Given a index of columns, the columns and the splitted line
    /// we return a hash of conditions AND the line itself.
    fn extract_conditions(
        &self,
        index_columns: &Vec<usize>,
        splitted_line: &Vec<&str>,
        columns_from_query: &Vec<String>,
    ) -> (Vec<(String, Value)>, Vec<String>) {
        let selected_columns = index_columns
            .iter()
            .map(|i| splitted_line[*i])
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let mut vec_conditions: Vec<(String, Value)> = Vec::new();

        // for each column, we need to map the condition
        for (j, _col) in selected_columns.iter().enumerate() {
            // with this we get the column that we want to check
            let column_condition = columns_from_query[j].as_str();
            // with this we get the value of the column
            let trimmed_value = selected_columns[j].trim().to_string();

            if let Some(v) = trimmed_value.parse::<i64>().ok() {
                vec_conditions.push((column_condition.to_string(), Value::Integer(v)));
                //hash_conditions.insert(column_condition.to_string(), Value::Integer(v));
            } else {
                vec_conditions.push((column_condition.to_string(), Value::String(trimmed_value)));
                //hash_conditions.insert(column_condition.to_string(), Value::String(trimmed_value));
            }
        }
        (vec_conditions, selected_columns)
    }

    pub fn insert_line_to_csv(&mut self, line: String) -> Result<(), std::io::Error> {
        // lets open the file name in append mode
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(self.file_name)?;

        file.write_all(line.as_bytes())?;
        Ok(())
    }

    pub fn resolve_delete(&mut self, conditions: Option<&str>) -> Result<(), std::io::Error> {
        // we need to check if the conditions are met
        // if they are met, we need to delete the line
        // else we need to keep the line
        // we need to write the lines that are not deleted in a temporal file
        // and then rename the temporal file to the original file
        // lets get the first line of the file to copy on the new file
        let columns_from_csv = std::io::BufReader::new(&self.file)
            .lines()
            .next()
            .unwrap_or(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error reading file",
            )))?;

        self.file.seek(SeekFrom::Start(0))?;

        let mut temporal_file = BufWriter::new(File::create("./temp_file.csv")?);
        temporal_file.write_all(columns_from_csv.as_bytes())?;

        for line in BufReader::new(&self.file).lines().into_iter() {
            let line = line?;
            let splitted_line = line.split(",").map(|s| s).collect::<Vec<&str>>();

            match conditions {
                Some(str_conditions) => {
                    let splitted_columns = columns_from_csv.split(",").collect::<Vec<&str>>();
                    let splitted_columns_as_string = splitted_columns
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();
                    let (hashed_conditions, _) = self.extract_conditions(
                        &(0..splitted_columns.len()).collect::<Vec<usize>>(),
                        &splitted_line,
                        &splitted_columns_as_string,
                    );

                    // lets print the conditions

                    // lets see all keys and values
                    let condition = Conditions::new(hashed_conditions);

                    // lets see all keys and values
                    if !condition.matches_condition(str_conditions) {
                        // criteria reached, we need to change the index
                        // of the columns according to the hash database with the proper value
                        temporal_file.write_all(line.as_bytes())?;
                        temporal_file.write_all("\n".as_bytes())?;
                    }
                }
                None => {
                    // we do basically nothing
                }
            }
        }

        Ok(())
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
        let result = table.resolve_select(columns, conditions, None);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn invalid_column_when_sorting() {
        let mut table = Table::new("./test.csv").unwrap();

        // tesis is the invalid columns
        let columns = vec!["Nombre".to_string(), "Edad".to_string()];

        let conditions: Option<&str> = None;

        let sorting = Some(vec![("Profesion".to_string(), true)]);

        // at t his point, we have this consult.
        // SELECT Nombre, Edad FROM test ORDER BY Profesion;
        // so we are trying to sort by a column that does not exist
        let result = table.resolve_select(columns, conditions, sorting);
        assert_eq!(result.is_err(), true);
    }
}

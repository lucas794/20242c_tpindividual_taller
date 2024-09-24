use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
};

use crate::{
    conditions::{condition::Condition, value::Value},
    sorter::sort::SortMethod,
};

use crate::errors::fileerrors::*;
use crate::errors::tperrors::*;

pub struct Table {
    file_name: String,
    file: File,
}

// lets implement a comparator to sort a vector

impl Table {
    pub fn new(path_table: String) -> Result<Self, std::io::Error> {
        let file_reference = File::open(&path_table);

        match file_reference {
            Ok(file) => Ok(Table {
                file,
                file_name: path_table.to_string(),
            }),
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

    pub fn get_file_directory(&self) -> String {
        self.file_name.to_string()
    }

    /// Get the file name of the table
    ///
    /// # Example
    ///
    /// ```./path/table.csv``` -> table
    ///
    /// ```./table.csv``` -> table
    pub fn get_file_name(&self) -> Result<String, Tperrors> {
        let table_file = match self.file_name.split("/").last() {
            Some(name) => name,
            None => {
                return Err(Tperrors::Generic(
                    "Error getting table file name by unknown reason".to_string(),
                ));
            }
        };

        // at this point i have table.csv, lets split again by . and get the first element
        let table_name = match table_file.split(".").next() {
            Some(name) => name,
            None => {
                return Err(Tperrors::Generic(
                    "Error getting table name, splitting by '.' failed".to_string(),
                ));
            }
        };
        Ok(table_name.to_string())
    }

    /// given the columns of table, conditions as str and a sorting method
    ///
    /// it will return the result of the query
    ///
    /// Example: Lets assume we have a DB with Nombre,Apellido
    /// ```SELECT Nombre FROM table WHERE Apellido = 'Doe' ORDER BY Nombre ASC;```
    ///
    /// The result will be a vector of vector of string (The content readed from the csv)

    pub fn resolve_select(
        &mut self,
        columns: Vec<String>,
        opt_conditions_as_str: Option<&str>,
        vector_sorting: Option<Vec<SortMethod>>,
    ) -> Result<Vec<Vec<String>>, std::io::Error> {
        // we need to match the index of the columns with the index of the csv
        // we need to read the csv and get the columns
        // we need to print the columns
        let columns_from_file = self.get_column_from_file()?;

        // we need to get the columns index, if the column isnt found, throw error
        // but it may be the joker (*) so we need to handle it
        let index_requested_columns = if columns.len() == 1 && columns[0] == "*" {
            (0..columns_from_file.len()).collect::<Vec<usize>>()
        } else {
            columns
                .iter()
                .map(|c| {
                    columns_from_file
                        .iter()
                        .position(|col| col == c)
                        .ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid column {} inside the query", c),
                            )
                        })
                })
                .collect::<Result<Vec<usize>, std::io::Error>>()?
        };

        self.file.seek(std::io::SeekFrom::Start(0))?; // lets place it after the columns name
        let mut result: Vec<Vec<String>> = Vec::new();

        for line_read in std::io::BufReader::new(&self.file).lines().skip(1) {
            let line = line_read?;
            let splitted_line = line.split(",").collect::<Vec<&str>>();

            if opt_conditions_as_str.is_some() {
                // we have conditions to check
                let (extracted_conditions, line_to_writte) = self.extract_conditions(
                    &(0..columns_from_file.len()).collect::<Vec<usize>>(),
                    &splitted_line,
                    &columns_from_file,
                );

                // we cut line_to_writte to keep only the index we requested
                let line_to_writte = index_requested_columns
                    .iter()
                    .map(|i| line_to_writte[*i].to_string())
                    .collect::<Vec<String>>();
                // now everything is clear and ready to check if conditions are met
                let condition = Condition::new(extracted_conditions);
                let str_conditions = opt_conditions_as_str.unwrap_or("");

                match condition.matches_condition(str_conditions) {
                    Ok(true) => {
                        result.push(line_to_writte);
                    }
                    Ok(false) => {
                        // we do nothing
                    }
                    Err(e) => {
                        let e = e.to_string();
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                    }
                }
            } else {
                // we need to push the matched columns to the vector
                let line_to_write = index_requested_columns
                    .iter()
                    .map(|i| splitted_line[*i])
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                result.push(line_to_write);
            }
        }

        if let Some(sorting) = vector_sorting {
            // first, let check if the columns are valid
            let columns_from_query = sorting
                .iter()
                .map(|method| method.get_by_column())
                .collect::<Vec<&String>>();
            if !columns_from_query.iter().all(|c| columns.contains(c)) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid columns inside the query",
                ));
            }

            // now sorting the vector
            result.sort_by(|a, b| {
                for method in &sorting {
                    let column = method.get_by_column();
                    let asc = method.is_ascending();

                    let index = columns.iter().position(|c| c == column).unwrap();
                    let a_value = a[index].as_str();
                    let b_value = b[index].as_str();

                    match a_value.cmp(b_value) {
                        Ordering::Less => {
                            if asc {
                                return Ordering::Less;
                            } else {
                                return Ordering::Greater;
                            }
                        }
                        Ordering::Greater => {
                            if asc {
                                return Ordering::Greater;
                            } else {
                                return Ordering::Less;
                            }
                        }
                        Ordering::Equal => {
                            continue;
                        }
                    }
                }
                Ordering::Equal
            });
        }
        // same as line_to_writte, we filter columns_from_file to keep only the requested columns
        let columns_from_file = index_requested_columns
            .iter()
            .map(|i| columns_from_file[*i].to_string())
            .collect::<Vec<String>>();

        result.insert(0, columns_from_file);
        Ok(result)
        /*
        let splitted_columns_from_file = match self.get_column_from_file() {
            Ok(columns) => columns,
            Err(e) => {
                return Err(e);
            }
        };

        // lets check if its a select *
        let index_columns = if columns.len() == 1 && columns[0] == "*" {
            (0..splitted_columns_from_file.len()).collect::<Vec<usize>>()
        } else {
            let temp_index = splitted_columns_from_file
                .iter()
                .enumerate()
                .filter(|(_i, c)| columns.contains(&c.to_string()))
                .map(|(i, _c)| i)
                .collect::<Vec<usize>>();

            if columns.len() != temp_index.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid columns inside the query",
                ));
            }
            temp_index
        };

        let columns = if columns.len() == 1 && columns[0] == "*" {
            // we need to handle the case if its a joker *
            splitted_columns_from_file
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
            let splitted_line = line.split(",").collect::<Vec<&str>>();

            if opt_conditions_as_str.is_some() {
                // we have conditions to check
                let (extracted_conditions, line_to_writte) =
                    self.extract_conditions(&index_columns, &splitted_line, &columns);

                println!("EC: {:?}", extracted_conditions);
                // now everything is clear and ready to check if conditions are met
                let condition = Condition::new(extracted_conditions);
                let str_conditions = opt_conditions_as_str.unwrap_or("");

                match condition.matches_condition(str_conditions) {
                    Ok(true) => {
                        result.push(line_to_writte);
                    }
                    Ok(false) => {
                        // we do nothing
                    }
                    Err(e) => {
                        let e = e.to_string();
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                    }
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
            let columns_from_query = sorting
                .iter()
                .map(|method| method.get_by_column())
                .collect::<Vec<&String>>();
            if !columns_from_query.iter().all(|c| columns.contains(c)) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid columns inside the query",
                ));
            }

            // now sorting the vector
            result.sort_by(|a, b| {
                for method in &sorting {
                    let column = method.get_by_column();
                    let asc = method.is_ascending();

                    let index = columns.iter().position(|c| c == column).unwrap();
                    let a_value = a[index].as_str();
                    let b_value = b[index].as_str();

                    match a_value.cmp(b_value) {
                        Ordering::Less => {
                            if asc {
                                return Ordering::Less;
                            } else {
                                return Ordering::Greater;
                            }
                        }
                        Ordering::Greater => {
                            if asc {
                                return Ordering::Greater;
                            } else {
                                return Ordering::Less;
                            }
                        }
                        Ordering::Equal => {
                            continue;
                        }
                    }
                }
                Ordering::Equal
            });
        }
        result.insert(0, columns); // to prevent clone, at the end the columns at the top of the vector.

        Ok(result)*/
    }

    /// given a columns and values as Vec of String
    ///
    /// It returns the proper line to write in the csv
    ///
    /// else returns a Error.
    ///
    pub fn resolve_insert(
        &self,
        columns: Vec<String>,
        values: Vec<String>,
    ) -> Result<Vec<String>, std::io::Error> {
        // we need to check if the columns are valid
        let splitted_columns_from_file = match self.get_column_from_file() {
            Ok(columns) => columns,
            Err(e) => {
                return Err(e);
            }
        };

        // if the column IS the same as values, this means that the columns weren't send on the query.
        let temp_index = if columns != values {
            splitted_columns_from_file
                .iter()
                .enumerate()
                .filter(|(_i, c)| columns.contains(&c.to_string()))
                .map(|(i, _c)| i)
                .collect::<Vec<usize>>()
        } else {
            (0..splitted_columns_from_file.len()).collect::<Vec<usize>>()
        };

        // columns != temp_index OR the table doesn't exist in the csv file.
        if columns.len() != temp_index.len() {
            println!("This is failing");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid columns",
            ));
        }

        // now we need to each temp_index, writ the value
        // else we write a empty string
        let mut line_to_write: Vec<String> = Vec::new();
        for (i, _col) in splitted_columns_from_file.iter().enumerate() {
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
    ///
    /// Given the columns to update, the values to update, and the conditions as str
    ///
    /// it will return the result of the query
    pub fn resolve_update(
        &mut self,
        columns: Vec<String>,
        values: Vec<String>,
        opt_conditions: Option<&str>,
    ) -> Result<(), std::io::Error> {
        // we need to check if the columns are valid
        let splitted_columns_from_file = match self.get_column_from_file() {
            Ok(columns) => columns,
            Err(e) => {
                return Err(e);
            }
        };

        let index_selected_column: Vec<usize> = splitted_columns_from_file
            .iter()
            .enumerate()
            .filter(|(_i, c)| columns.contains(&c.to_string()))
            .map(|(i, _c)| i)
            .collect::<Vec<usize>>();

        let index_all_columns = (0..splitted_columns_from_file.len()).collect::<Vec<usize>>();

        // we need to change the value of the columns
        // we use a hash to store the new values
        // and keys the index of the columns of change
        // the change is done if the conditions are met
        let mut hash_changes: HashMap<usize, String> = HashMap::new();

        for (i, _col) in splitted_columns_from_file.iter().enumerate() {
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

        // get current path where the file is located
        let formal_path = format!("{}/temporal_file.csv", self.get_directory_where_file_is());

        let mut temporal_file = BufWriter::new(File::create(formal_path)?);

        temporal_file.write_all(splitted_columns_from_file.join(",").as_bytes())?;

        temporal_file.write_all("\n".as_bytes())?;

        for line in BufReader::new(&self.file).lines().skip(1) {
            let line = line?;
            let splitted_line = line.split(",").collect::<Vec<&str>>();

            match opt_conditions {
                Some(str_conditions) => {
                    let splitted_columns_as_string = splitted_columns_from_file.as_slice();
                    let (vec_conditions, _) = self.extract_conditions(
                        &index_all_columns,
                        &splitted_line,
                        splitted_columns_as_string,
                    );

                    // lets see all keys and values
                    let condition = Condition::new(vec_conditions);

                    match condition.matches_condition(str_conditions) {
                        Ok(true) => {
                            // criteria reached, we need to change the index
                            // of the columns according to the hash database with the proper value
                            let mut new_line = splitted_line.to_vec();
                            for (i, value) in hash_changes.iter() {
                                new_line[*i] = value;
                            }
                            // convert it as Vec<String

                            temporal_file.write_all(new_line.join(",").as_bytes())?;
                            temporal_file.write_all("\n".as_bytes())?;
                        }
                        Ok(false) => {
                            temporal_file.write_all(splitted_line.join(",").as_bytes())?;
                            temporal_file.write_all("\n".as_bytes())?;
                        }
                        Err(_) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Error checking conditions".to_string(),
                            ));
                        }
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

    /// Helper to extract the conditions from the splitted line
    ///
    /// Given a index of columns, the columns and the splitted line
    ///
    /// we return a hash of conditions AND the line itself.
    ///
    fn extract_conditions(
        &self,
        index_columns: &[usize],
        splitted_line: &[&str],
        columns_from_query: &[String],
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

            if let Ok(v) = trimmed_value.parse::<i64>() {
                vec_conditions.push((column_condition.to_string(), Value::Integer(v)));
            } else {
                vec_conditions.push((column_condition.to_string(), Value::String(trimmed_value)));
            }
        }
        (vec_conditions, selected_columns)
    }

    /// Function that handles the insert query
    ///
    /// Given a line, we writte it on the 'database' (our csv file)
    pub fn insert_line_to_csv(&mut self, line: String) -> Result<(), std::io::Error> {
        // lets open the file name in append mode
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.file_name)?;

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
        let splitted_columns_from_file = match self.get_column_from_file() {
            Ok(columns) => columns,
            Err(e) => {
                return Err(e);
            }
        };
        let columns_from_csv = splitted_columns_from_file.join(",");

        self.file.seek(SeekFrom::Start(0))?;

        let formal_path = format!("{}/temporal_file.csv", self.get_directory_where_file_is());

        let mut temporal_file = BufWriter::new(File::create(formal_path)?);

        temporal_file.write_all(columns_from_csv.as_bytes())?;

        for line in BufReader::new(&self.file).lines() {
            let line = line?;
            let splitted_line = line.split(",").collect::<Vec<&str>>();

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
                    // lets see all keys and values
                    let condition = Condition::new(hashed_conditions);

                    // lets see all keys and values
                    match condition.matches_condition(str_conditions) {
                        Ok(true) => {
                            temporal_file.write_all(line.as_bytes())?;
                            temporal_file.write_all("\n".as_bytes())?;
                        }
                        Ok(false) => {
                            // criteria reached, we need to change the index
                            // of the columns according to the hash database with the proper value
                            /*temporal_file.write_all(line.as_bytes())?;
                            temporal_file.write_all("\n".as_bytes())?;*/
                        }
                        Err(_) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Error checking conditions",
                            ));
                        }
                    }
                }
                None => {
                    // we do basically nothing
                }
            }
        }

        Ok(())
    }

    /// Returns the directory where the file is located
    /// Example: ./path/to/file.csv -> ./path/to
    /// Example: ./file.csv -> ./
    pub fn get_directory_where_file_is(&self) -> String {
        let file_path_pos = self.get_file_directory().rfind('/').unwrap_or(0);

        match file_path_pos {
            0 => "./".to_string(),
            _ => self.get_file_directory()[..file_path_pos].to_string(),
        }
    }

    /// The approach in this work is to avoid reading the whole line on memory.
    /// So, we create a "temp" csv file with the output
    /// Then, at the end, switch names.
    pub fn replace_original_with_tempfile(&self) -> Result<(), FileErrors> {
        let original_file = self.get_file_directory();
        let replacement = format!("{}/temporal_file.csv", self.get_directory_where_file_is());

        match fs::remove_file(&original_file) {
            Ok(_) => {}
            Err(_) => {
                return Err(FileErrors::DeletionFailed);
            }
        }

        match fs::rename(replacement, &original_file) {
            Ok(_) => Ok(()),
            Err(_) => Err(FileErrors::InvalidFile),
        }
    }

    /// gets the columns of the table as string
    fn get_column_from_file(&self) -> Result<Vec<String>, std::io::Error> {
        let columns = std::io::BufReader::new(&self.file)
            .lines()
            .next()
            .unwrap_or(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error reading file",
            )))?;

        let splitted_columns = columns.split(",").collect::<Vec<&str>>();
        Ok(splitted_columns.iter().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let table = Table::new("./tests/data/database.csv".to_string()).unwrap();
        let filename = table.get_file_name().unwrap();
        assert_eq!(filename, "database");
    }

    #[test]
    fn invalid_table() {
        let invalid_routes = vec!["./invalidtable.csv", "./invalidtable"];

        for invalid_route in invalid_routes {
            let table = Table::new(invalid_route.to_string());
            assert_eq!(table.is_err(), true);
        }
    }

    #[test]
    fn invalid_column() {
        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();

        // tesis is the invalid columns
        let columns = vec!["Edad".to_string(), "Tesis".to_string()];
        let conditions = Some("WHERE name = 'John'");
        let result = table.resolve_select(columns, conditions, None);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn invalid_column_when_sorting() {
        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();

        // tesis is the invalid columns
        let columns = vec!["Nombre".to_string(), "Edad".to_string()];

        let conditions: Option<&str> = None;

        let sorting = Some(vec![SortMethod {
            by_column: "Profesion".to_string(),
            ascending: true,
        }]);

        // at t his point, we have this consult.
        // SELECT Nombre, Edad FROM test ORDER BY Profesion;
        // so we are trying to sort by a column that does not exist
        let result = table.resolve_select(columns, conditions, sorting);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn return_select_returns_ok() {
        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();

        let columns = vec!["Nombre".to_string(), "Edad".to_string()];
        let result = table.resolve_select(columns, None, None);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn return_select_returns_ok_with_conditions() {
        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();

        let columns = vec!["Nombre".to_string(), "Edad".to_string()];
        let conditions = Some("Nombre = 'Luis' AND Edad = 29");
        let result = table.resolve_select(columns, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Edad".to_string()],
            vec!["Luis".to_string(), "29".to_string()],
        ];

        assert_eq!(result, expected_result);
    }

    #[test]
    fn return_select_returns_proper_order_of_requested_columns() {
        // I'm trying to do a SELECT Edad, Nombre FROM table WHERE Edad = 45;
        // Edad = 45 only to get one result.

        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();
        let columns = vec!["Edad".to_string(), "Nombre".to_string()];
        let conditions = Some("Edad = 45");
        let sorting = None;

        // execute_Selects do a print, so we need to hook it

        let expected_result = vec![
            vec!["Edad".to_string(), "Nombre".to_string()],
            vec!["45".to_string(), "Carlos".to_string()],
        ];

        let result = table.resolve_select(columns, conditions, sorting).unwrap();

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }
    #[test]
    fn return_select_returns_proper_answer_without_passing_the_column_as_query() {
        // I'm trying to do a SELECT Nombre FROM table WHERE Edad = 45;
        // So i'm going to get only Carlos as result.

        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();
        let columns = vec!["Nombre".to_string()];
        let conditions = Some("Edad = 45");
        let sorting = None;

        // execute_Selects do a print, so we need to hook it

        let expected_result = vec![vec!["Nombre".to_string()], vec!["Carlos".to_string()]];

        let result = table.resolve_select(columns, conditions, sorting).unwrap();

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }

    #[test]
    fn return_select_returns_proper_answer_with_column_with_spaces() {
        // I'm trying to do a SELECT \"Correo electronico\" FROM table WHERE Edad = 45;
        // So i'm going to get only csanchez@gmail.com as result.

        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();
        let columns = vec!["Correo electronico".to_string()];
        let conditions = Some("Edad = 45");
        let sorting = None;

        // execute_Selects do a print, so we need to hook it

        let expected_result = vec![
            vec!["Correo electronico".to_string()],
            vec!["csanchez@gmail.com".to_string()],
        ];

        let result = table.resolve_select(columns, conditions, sorting).unwrap();

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }
    #[test]
    fn return_select_returns_ok_with_nested_parenthesis_condition() {
        let mut table = Table::new("./tests/data/database.csv".to_string()).unwrap();

        let columns = vec!["Nombre".to_string(), "Profesion".to_string()];
        let conditions = Some("(Edad >= 32 AND Edad <= 40) AND (Nombre = Juan OR Nombre = Pedro)");
        let result = table.resolve_select(columns, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Profesion".to_string()],
            vec!["Juan".to_string(), "medico".to_string()],
            vec!["Pedro".to_string(), "diseñador".to_string()],
        ];

        assert_eq!(result, expected_result);
    }
}

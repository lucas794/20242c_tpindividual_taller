use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    conditions::{condition::Condition, value::Value},
    sorter::sort::SortMethod,
};

use crate::errors::fileerrors::*;
use crate::errors::tperrors::*;

pub struct Table<R: Read + Seek> {
    file_name: String,
    reader: BufReader<R>,
}

impl<R: Read + Seek> Table<R> {
    /// Mock a table with a file name and data
    ///
    /// This is used for testing purposes
    pub fn mock(file_name: String, data: &'static [u8]) -> Table<Cursor<&'static [u8]>> {
        let cursor = Cursor::new(data as &[u8]);
        let reader = BufReader::new(cursor);
        Table { file_name, reader }
    }

    pub fn new(path_table: String) -> Result<Table<File>, std::io::Error> {
        let file_reference = File::open(&path_table)?;

        Ok(Table {
            file_name: path_table,
            reader: BufReader::new(file_reference),
        }) // lets close the file
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
    ) -> Result<Vec<Vec<String>>, Tperrors> {
        let columns_from_file = self.get_column_from_file()?;

        // if len is 1 AND the only element is a * (joker) we need to get all the columns
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
                            Tperrors::Column(format!("Invalid column {} inside the query", c))
                        })
                })
                .collect::<Result<Vec<usize>, Tperrors>>()?
        };

        let mut result: Vec<Vec<String>> = Vec::new();

        match self.reader.seek(SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to read the file: {}",
                    e
                )));
            }
        }
        let reader = &mut self.reader;

        let index_columns = (0..columns_from_file.len()).collect::<Vec<usize>>();
        for line in reader.by_ref().lines().skip(1) {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    return Err(Tperrors::Generic(format!(
                        "Error while trying to read the file: {}",
                        e
                    )));
                }
            };
            let splitted_line = line.split(",").collect::<Vec<&str>>();

            if opt_conditions_as_str.is_some() {
                let (extracted_conditions, _line_to_write) =
                    Self::extract_conditions(&index_columns, &splitted_line, &columns_from_file);
                //panic!("Extracted conditions: {:?}", extracted_conditions);
                let condition = Condition::new(extracted_conditions);
                let str_conditions = opt_conditions_as_str.unwrap_or("");

                match condition.matches_condition(str_conditions) {
                    Ok(true) => {
                        result.push(splitted_line.iter().map(|s| s.to_string()).collect());
                    }
                    Ok(false) => {}
                    Err(e) => {
                        let e = e.to_string();
                        return Err(Tperrors::Generic(e));
                    }
                }
            } else {
                result.push(splitted_line.iter().map(|s| s.to_string()).collect());
            }
        }

        // at this point, i have the result of the query
        // I need to sort it as needed, and now keep only the columns requested
        if let Some(vec_sort) = vector_sorting {
            for sort_method in &vec_sort {
                let column = sort_method.get_by_column();
                let index = columns_from_file
                    .iter()
                    .position(|c| c == column)
                    .ok_or_else(|| {
                        Tperrors::Column(format!("Invalid column {} inside the query", column))
                    })?;

                result.sort_by(|a, b| {
                    let a_value = a[index].as_str();
                    let b_value = b[index].as_str();

                    match a_value.cmp(b_value) {
                        Ordering::Less => {
                            if sort_method.is_ascending() {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            }
                        }
                        Ordering::Greater => {
                            if sort_method.is_ascending() {
                                Ordering::Greater
                            } else {
                                Ordering::Less
                            }
                        }
                        Ordering::Equal => Ordering::Equal,
                    }
                });
            }
        }

        // last thing, we filter the columns requested
        result = result
            .iter()
            .map(|line| {
                index_requested_columns
                    .iter()
                    .map(|i| line[*i].to_string())
                    .collect()
            })
            .collect();

        // lets only now keep the headers of the columns requested
        let header_requested = index_requested_columns
            .iter()
            .map(|i| columns_from_file[*i].to_string())
            .collect::<Vec<String>>();

        result.insert(0, header_requested); // we add at the head the columns of the db
        Ok(result)
    }

    /// given a columns and values as Vec of String
    ///
    /// It returns a vector of lines to add to the file.
    ///
    /// else returns a Error.
    ///
    pub fn resolve_insert(
        &mut self,
        columns: Vec<String>,
        values: Vec<Vec<String>>,
    ) -> Result<Vec<Vec<String>>, Tperrors> {
        // we need to check if the columns are valid
        let splitted_columns_from_file = self.get_column_from_file()?;

        // if any of the vec inside values matches column, we are sending all values
        let temp_index = if columns
            .iter()
            .all(|c| splitted_columns_from_file.contains(c))
        {
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
            return Err(Tperrors::Column(
                "Invalid column inside the query".to_string(),
            ));
        }

        // now we need to each temp_index, writ the value
        // else we write a empty string
        // Prepare the line to write, matching table columns
        let mut vector_of_lines_to_writte: Vec<Vec<String>> = Vec::new();

        for value in values {
            // So now we have either a full value like (X,X,X,X,X,X)
            // or (X,X), we need to find if the len is full, if it is
            // then we need to join all values, if not, we need to replace
            // the missing places with ""
            let mut temporal_line_to_write = Vec::new();
            if value.len() == splitted_columns_from_file.len() {
                temporal_line_to_write.push(value.join(","));
            } else {
                // we need to iter over all columns found inside the file.
                // if the position i of the column is inside temp_index
                // it means that the value must be written
                for (i, _col) in splitted_columns_from_file.iter().enumerate() {
                    if temp_index.contains(&i) {
                        // we need the index reference on columns
                        let reference = match temp_index.iter().position(|&x| x == i) {
                            Some(p) => p,
                            None => {
                                return Err(Tperrors::Generic(
                                    "Position not found inside the columns".to_string(),
                                ));
                            }
                        };
                        temporal_line_to_write.push(value[reference].to_string());
                    } else {
                        temporal_line_to_write.push("".to_string());
                    }
                }
            }
            vector_of_lines_to_writte.push(temporal_line_to_write);
        }

        Ok(vector_of_lines_to_writte)
    }

    /// Private function that handles
    ///
    /// Given the columns to update, the values to update, and the conditions as str
    ///
    /// it will resolve the query, if ok it will return Ok() else it will throw an error
    ///
    /// containing the result of the query.
    fn resolve_update<W: Write>(
        &mut self,
        columns: Vec<String>,
        values: Vec<String>,
        opt_conditions: Option<&str>,
        file_to_write: W, // its either a Cursor o a File as temp
    ) -> Result<(), Tperrors> {
        // we need to check if the columns are valid
        let splitted_columns_from_file = self.get_column_from_file()?;

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
                        return Err(Tperrors::Generic(
                            "Position not found inside the columns".to_string(),
                        ));
                    }
                };
                hash_changes.insert(i, values[position].to_string());
            }
        }

        match self.reader.seek(SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to read the file: {}",
                    e
                )));
            }
        }

        let mut temporal_file = BufWriter::new(file_to_write);

        match Self::write_a_line(splitted_columns_from_file.join(","), &mut temporal_file) {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to write the file: {}",
                    e
                )));
            }
        }

        for line in self.reader.by_ref().lines().skip(1) {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    return Err(Tperrors::Generic(format!(
                        "Error while trying to read the file: {}",
                        e
                    )));
                }
            };
            let splitted_line = line.split(",").collect::<Vec<&str>>();

            match opt_conditions {
                Some(str_conditions) => {
                    let splitted_columns_as_string = splitted_columns_from_file.as_slice();
                    let (vec_conditions, _) = Self::extract_conditions(
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

                            match Self::write_a_line(new_line.join(","), &mut temporal_file) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(Tperrors::Generic(format!(
                                        "Error while trying to write the file: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Ok(false) => {
                            match Self::write_a_line(splitted_line.join(","), &mut temporal_file) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(Tperrors::Generic(format!(
                                        "Error while trying to write the file: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Err(_) => {
                            return Err(Tperrors::Generic("Error checking conditions".to_string()));
                        }
                    }
                }
                None => {
                    // we need to change the values
                    let mut new_line = splitted_line.to_vec();
                    for (i, value) in hash_changes.iter() {
                        new_line[*i] = value;
                    }
                    match Self::write_a_line(new_line.join(","), &mut temporal_file) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(Tperrors::Generic(format!(
                                "Error while trying to write the file: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }
        match temporal_file.flush() {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to write the file: {}",
                    e
                )));
            }
        }
        Ok(())
    }

    /// Function to resolve the update query
    ///
    /// Under a file path, it will resolve the update query
    ///
    /// It will return the path of the temporal file to make later make the switch
    ///
    pub fn resolve_update_for_file(
        &mut self,
        columns: Vec<String>,
        values: Vec<String>,
        opt_conditions: Option<&str>,
    ) -> Result<String, Tperrors> {
        let temporal_file_path = match self.generate_temporal_file_path() {
            Ok(path) => path,
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to generate temporal file: {}",
                    e
                )));
            }
        };
        let temporal_file = match File::create(&temporal_file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to create temporal file: {}",
                    e
                )));
            }
        };
        self.resolve_update(columns, values, opt_conditions, temporal_file)?;
        Ok(temporal_file_path)
    }

    pub fn resolve_update_mock(
        &mut self,
        columns: Vec<String>,
        values: Vec<String>,
        opt_conditions: Option<&str>,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, Tperrors> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufWriter::new(cursor);

        self.resolve_update(columns, values, opt_conditions, &mut writer)?;

        let inner_buffer = match writer.into_inner() {
            Ok(b) => b.into_inner(),
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to write the file: {}",
                    e
                )));
            }
        };
        Ok(BufReader::new(Cursor::new(inner_buffer)))
    }

    /// Helper to extract the conditions from the splitted line
    ///
    /// Given a index of columns, the columns and the splitted line
    ///
    /// we return a hash of conditions AND the line itself.
    ///
    fn extract_conditions(
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

        file.write_all('\n'.to_string().as_bytes())?;
        file.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Internal function to write a line
    fn write_a_line<W: Write>(line: String, file: &mut W) -> Result<(), std::io::Error> {
        match file.write_all(line.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        };
        match file.write_all('\n'.to_string().as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Internal function that resolves a delete operatior
    ///
    /// given a condition and a writter (file or cursor)
    ///
    /// it will write the result on the writer and return Ok() if everything is ok
    ///
    /// If it fails it will throw a error from std::io::Error
    fn resolve_delete<W: Write>(
        &mut self,
        conditions: Option<&str>,
        file: W,
    ) -> Result<(), Tperrors> {
        // we need to check if the conditions are met
        // if they are met, we need to delete the line
        // else we need to keep the line
        // we need to write the lines that are not deleted in a temporal file
        // and then rename the temporal file to the original file
        // lets get the first line of the file to copy on the new file
        let splitted_columns_from_file = self.get_column_from_file()?;

        let columns_from_csv = splitted_columns_from_file.join(",");
        match self.reader.seek(SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to read the file: {}",
                    e
                )));
            }
        }

        let mut temporal_file = BufWriter::new(file);

        match Self::write_a_line(columns_from_csv, &mut temporal_file) {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to write the file: {}",
                    e
                )));
            }
        }

        for line in self.reader.by_ref().lines().skip(1) {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    return Err(Tperrors::Generic(format!(
                        "Error while trying to read the file: {}",
                        e
                    )));
                }
            };
            let splitted_line = line.split(",").collect::<Vec<&str>>();

            match conditions {
                Some(str_conditions) => {
                    let splitted_columns_as_string = splitted_columns_from_file
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();

                    let (extracted_conditions, _line_to_write) = Self::extract_conditions(
                        &(0..splitted_columns_as_string.len()).collect::<Vec<usize>>(),
                        &splitted_line,
                        &splitted_columns_as_string,
                    );
                    let condition = Condition::new(extracted_conditions);
                    match condition.matches_condition(str_conditions) {
                        Ok(true) => {
                            // critera matches? we do nothing
                        }
                        Ok(false) => {
                            // criteria reached, we need to change the index
                            // of the columns according to the hash database with the proper value

                            match Self::write_a_line(line, &mut temporal_file) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(Tperrors::Generic(format!(
                                        "Error while trying to write the file: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Err(_) => {
                            return Err(Tperrors::Generic("Error checking conditions".to_string()));
                        }
                    }
                }
                None => {
                    // we do basically nothing
                }
            }
        }
        match temporal_file.flush() {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to write the file: {}",
                    e
                )));
            }
        }
        Ok(())
    }

    /// Function that resolves the delete query
    ///
    /// Given a condition, it will return the path of the temporal file
    ///
    /// If it fails it will throw a error from std::io::Error
    pub fn resolve_delete_for_file(
        &mut self,
        conditions: Option<&str>,
    ) -> Result<String, Tperrors> {
        let temporal_file_path = match self.generate_temporal_file_path() {
            Ok(path) => path,
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to generate temporal file: {}",
                    e
                )));
            }
        };
        let temporal_file = match File::create(&temporal_file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to create temporal file: {}",
                    e
                )));
            }
        };
        self.resolve_delete(conditions, temporal_file)?;
        Ok(temporal_file_path)
    }

    /// Mock function that resolves the delete query
    ///
    /// Given a condition, it will return a BufReader with the result of the query
    ///
    /// If it fails it will throw a error from std::io::Error
    pub fn resolve_delete_mock(
        &mut self,
        conditions: Option<&str>,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, Tperrors> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufWriter::new(cursor);

        self.resolve_delete(conditions, &mut writer)?;

        let inner_buffer = match writer.into_inner() {
            Ok(b) => b.into_inner(),
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to write the file: {}",
                    e
                )));
            }
        };
        Ok(BufReader::new(Cursor::new(inner_buffer)))
    }

    /// Returns the directory where the file is located
    /// Example: ./path/to/file.csv -> ./path/to
    /// Example: ./file.csv -> ./
    pub fn get_directory_where_file_is(&self) -> String {
        let file_path_pos = self.get_file_directory().rfind('/').unwrap_or(0);

        match file_path_pos {
            0 => ".".to_string(),
            _ => self.get_file_directory()[..file_path_pos].to_string(),
        }
    }

    /// The approach in this work is to avoid reading the whole line on memory.
    /// So, we create a "temp" csv file with the output
    /// Then, at the end, switch names.
    pub fn replace_original_with(&self, temporal_file: String) -> Result<(), FileErrors> {
        let original_file = self.get_file_directory();

        match fs::remove_file(&original_file) {
            Ok(_) => {}
            Err(_) => return Err(FileErrors::DeletionFailed),
        }

        // now we rename the temporal file to the original file
        match fs::rename(temporal_file, &original_file) {
            Ok(_) => {}
            Err(_) => return Err(FileErrors::InvalidFile),
        }
        Ok(())
    }

    /// gets the columns of the table as string
    fn get_column_from_file(&mut self) -> Result<Vec<String>, Tperrors> {
        match self.reader.seek(SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(e) => {
                return Err(Tperrors::Generic(format!(
                    "Error while trying to read the file: {}",
                    e
                )));
            }
        }

        let first_column = match self.reader.by_ref().lines().next() {
            Some(line) => match line {
                Ok(l) => l,
                Err(e) => {
                    return Err(Tperrors::Generic(format!(
                        "Error while trying to read the file: {}",
                        e
                    )));
                }
            },
            None => {
                return Err(Tperrors::Column(
                    "Error while trying to read the file: No columns found".to_string(),
                ));
            }
        };
        let splitted_columns = first_column.split(",").collect::<Vec<&str>>();
        Ok(splitted_columns.iter().map(|s| s.to_string()).collect())
    }

    /// Generates a temporal file path
    ///
    /// It will return a string with the path of the temporal file
    ///
    /// If it fails it will throw a error from std::io::Error
    pub fn generate_temporal_file_path(&self) -> Result<String, std::io::Error> {
        let start = SystemTime::now();
        let since_the_epoch = match start.duration_since(UNIX_EPOCH) {
            Ok(time) => time,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error getting time",
                ));
            }
        };
        Ok(format!(
            "{}/temporal_file_{}.csv",
            self.get_directory_where_file_is(),
            since_the_epoch.as_micros()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV_DATA: &str = "Id,Nombre,Apellido,Edad,Correo electronico,Profesion\n\
    1,Juan,Perez,32,jperez@gmail.com,medico\n\
    2,Maria,Gomez,28,mgomez@gmail.com,abogado\n\
    3,Carlos,Sánchez,45,csanchez@gmail.com,ingeniero\n\
    4,Ana,Ruiz,36,aruiz@gmail.com,arquitecta\n\
    5,Luis,Martínez,29,lmartinez@gmail.com,profesor\n\
    6,Laura,Domínguez,41,ldominguez@gmail.com,enfermera\n\
    7,Pedro,Fernández,33,pfernandez@gmail.com,diseñador\n\
    8,Lucía,Ramos,26,lramos@gmail.com,psicóloga\n\
    9,Diego,Navarro,39,dnavarro@gmail.com,empresario\n\
    10,Paula,Hernández,31,phernandez@gmail.com,publicista\n\
    11,Andrés,García,34,andresgarcia@gmail.com,contador y ingeniero\n\
    ";

    #[test]
    fn new() {
        let table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());
        //let table = Table::<File>::new("./tests/data/database.csv".to_string()).unwrap();
        let filename = table.get_file_name().unwrap();
        assert_eq!(filename, "database");
    }

    #[test]
    fn invalid_table() {
        let invalid_routes = vec!["./invalidtable.csv", "./invalidtable"];

        for invalid_route in invalid_routes {
            let table = Table::<File>::new(invalid_route.to_string());
            assert_eq!(table.is_err(), true);
        }
    }

    #[test]
    fn invalid_column() {
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        // tesis is the invalid columns
        let columns = vec!["Edad".to_string(), "Tesis".to_string()];
        let conditions = Some("WHERE name = 'John'");
        let result = table.resolve_select(columns, conditions, None);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn invalid_column_when_sorting() {
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        // tesis is the invalid columns
        let columns = vec!["Nombre".to_string(), "Edad".to_string()];

        let conditions: Option<&str> = None;

        let sorting = Some(vec![SortMethod {
            by_column: "Trabajo Profesional".to_string(),
            ascending: true,
        }]);

        // at t his point, we have this consult.
        // SELECT Nombre, Edad FROM test ORDER BY Trabajo Profesional;
        // so we are trying to sort by a column that does not exist
        let result = table.resolve_select(columns, conditions, sorting);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_select_returns_ok() {
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let columns = vec!["Nombre".to_string(), "Edad".to_string()];
        let result = table.resolve_select(columns, None, None);
        println!("{:?}", result);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_select_returns_ok_with_conditions() {
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let columns = vec!["Nombre".to_string(), "Edad".to_string()];
        let conditions = Some("Nombre = 'Luis' AND Edad>15");
        let result = table.resolve_select(columns, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Edad".to_string()],
            vec!["Luis".to_string(), "29".to_string()],
        ];

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_select_returns_proper_order_of_requested_columns() {
        // I'm trying to do a SELECT Edad, Nombre FROM table WHERE Edad = 45;
        // Edad = 45 only to get one result.

        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());
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
    fn test_select_returns_proper_answer_without_passing_the_column_as_query() {
        // I'm trying to do a SELECT Nombre FROM table WHERE Edad = 45;
        // So i'm going to get only Carlos as result.

        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());
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
    fn test_select_returns_proper_answer_with_column_with_spaces() {
        // I'm trying to do a SELECT \"Correo electronico\" FROM table WHERE Edad = 45;
        // So i'm going to get only csanchez@gmail.com as result.

        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());
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
    fn test_select_returns_ok_with_nested_parenthesis_condition() {
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let columns = vec!["Nombre".to_string(), "Profesion".to_string()];
        let conditions = Some("(Edad >= 32 AND Edad <= 40) AND (Nombre = Juan OR Nombre = Pedro)");
        let result = table.resolve_select(columns, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Profesion".to_string()],
            vec!["Juan".to_string(), "medico".to_string()],
            vec!["Pedro".to_string(), "diseñador".to_string()],
        ];

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }

    #[test]
    fn test_select_returns_ok_with_conditions_attached() {
        // We are trying to simulate a
        // SELECT Nombre, Edad FROM clientes WHERE Edad>=45 AND Edad<=43;
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let columns = vec!["Nombre".to_string(), "Edad".to_string()]; // SELECT ALL
        let conditions = Some("Edad>=41 AND Edad<=43");
        let result = table.resolve_select(columns, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Edad".to_string()],
            vec!["Laura".to_string(), "41".to_string()],
        ];

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }

    #[test]
    fn test_select_returns_ok_with_conditions_attached_desbalanced() {
        // We are trying to simulate a
        // SELECT Nombre, Edad FROM clientes WHERE Edad>=45 AND Edad <= 43;
        // conditions are desbalanced and separated, it should work anyway
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let columns = vec!["Nombre".to_string(), "Edad".to_string()]; // SELECT ALL
        let conditions = Some("Edad>=41 AND Edad <= 43");
        let result = table.resolve_select(columns, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Edad".to_string()],
            vec!["Laura".to_string(), "41".to_string()],
        ];

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }

    #[test]
    fn test_select_general_column_sorting_without_a_column_presents_in_the_query_returns_ok() {
        // We are trying to simulate a
        // SELECT * FROM clientes ORDER BY nombre ASC;
        // So we're trying to sort by a column that is not present in the query.
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let column = vec!["*".to_string()];

        let ordering = Some(vec![SortMethod {
            by_column: "Nombre".to_string(),
            ascending: true,
        }]);

        let result = table.resolve_select(column, None, ordering);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_select_certain_column_not_present_in_query_and_sort_with_other_returns_ok() {
        // Trying to do
        // SELECT apellido FROM clientes ORDER BY nombre DESC;
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let column = vec!["Apellido".to_string()];
        let ordering = Some(vec![SortMethod {
            by_column: "Nombre".to_string(),
            ascending: false,
        }]);

        let result = table.resolve_select(column, None, ordering);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_select_without_finishing_condition_operator_throws_err() {
        // We are trying to simulate a
        // SELECT * FROM clientes WHERE Edad = 45 AND;
        // So we are trying to finish the condition with an operator.
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let column = vec!["*".to_string()];
        let conditions = Some("Edad=45 AND");
        let result = table.resolve_select(column, conditions, None);

        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_select_with_values_as_scaped_value_returns_ok() {
        // We are trying to simulate a
        // SELECT * FROM clientes WHERE Profesion = 'contador + ingeniero';
        // So we are trying to scape the value of the condition.
        let mut table = Table::<Cursor<&[u8]>>::mock("database".to_string(), CSV_DATA.as_bytes());

        let column = vec!["Nombre".to_string(), "Edad".to_string()];
        let conditions = Some("Profesion='contador y ingeniero'");
        let result = table.resolve_select(column, conditions, None).unwrap();

        let expected_result = vec![
            vec!["Nombre".to_string(), "Edad".to_string()],
            vec!["Andrés".to_string(), "34".to_string()],
        ];

        for (i, line) in result.iter().enumerate() {
            assert_eq!(line, &expected_result[i]);
        }
    }
}

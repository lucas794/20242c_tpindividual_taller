use std::{
    fs::File,
    io::{BufRead, Read, Seek},
};

pub struct Table {
    file_name: String,
    file: File,
}

impl Table {
    pub fn new(file_name: &String) -> Result<Self, std::io::Error> {
        let file_reference = File::open(file_name);

        match file_reference {
            Ok(file) => Ok(Table {
                file: file,
                file_name: file_name.to_string(),
            }),
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

    pub fn get_file_name(&self) -> String {
        // table name can be ./path/table.csv
        // or ./table.csv
        // or table.csv
        // idea: split by /, get the last element, remove csv.

        let table_file = match self.file_name.split("/").last() {
            Some(name) => name,
            None => {
                println!("Error getting table file name by unknown reason");
                std::process::exit(1);
            }
        };

        // at this point i have table.csv, lets split again by . and get the first element
        let table_name = match table_file.split(".").next() {
            Some(name) => name,
            None => {
                println!("Error getting table name, splitting by '.' failed");
                std::process::exit(1);
            }
        };
        table_name.to_string()
    }

    pub fn execute_select(&mut self, columns: Vec<String>) -> Result<Vec<Vec<String>>, std::io::Error> {
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

        println!("Index columns inside file: {:?}", splitted_columns);
        println!("splitted_columns: {:?}", columns);

        let index_columns = splitted_columns
            .iter()
            .enumerate()
            .filter(|(i, c)| columns.contains(&c.to_string()))
            .map(|(i, c)| i)
            .collect::<Vec<usize>>();

        println!("Index columns: {:?}", index_columns);

        if columns.len() != index_columns.len() {
            println!("[INVALID_SYNTAX]: There are invalid columns inside the query");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid columns",
            ));
        }

        // lets read the file line by line
        // and print the columns
        println!("Reading file... {}", self.file_name);
        self.file.seek(std::io::SeekFrom::Start(0))?;
        let mut result: Vec<Vec<String>> = Vec::new();

        // adding columns to the result
        result.push(columns);

        for line in std::io::BufReader::new(&self.file).lines().skip(1) {
            let line = line?;
            let splitted_line = line.split(",").collect::<Vec<&str>>();
            let selected_columns = index_columns
                .iter()
                .map(|i| splitted_line[*i].to_string())
                .collect::<Vec<String>>();

            result.push(selected_columns);
        }

        Ok(result)
    }
}

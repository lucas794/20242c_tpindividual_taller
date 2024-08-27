use std::fs::File;

pub struct Table {
    file_name: String,
    file: File,
}

impl Table {
    pub fn new(file_name: &String) -> Self {
        let file_reference = File::open(file_name);

        match file_reference {
            Ok(file) => Table {
                file: file,
                file_name: file_name.to_string(),
            },
            Err(e) => {
                println!("[INVALID_TABLE]: Error {}", e);
                std::process::exit(1);
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
}

use std::{collections::HashMap, fs, io, rc::Rc};

/// FolderTables is a struct that contains a HashMap
///
/// With the table name as String, and the path to the table as String
pub struct FolderTables {
    data: HashMap<String, String>,
}

impl FolderTables {
    pub fn new(path_folder: &str) -> Result<FolderTables, io::Error> {
        let folder = fs::read_dir(path_folder)?;

        let mut temp_hash: HashMap<String, String> = HashMap::new();

        for file in folder {
            // we get the file name only, without extension
            let file = match file {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let rc = Rc::new(file);

            let file_name_ref = Rc::clone(&rc);
            let file_name = match file_name_ref.file_name().into_string() {
                Ok(name) => {
                    let find_dot = match name.rfind(".") {
                        Some(dot) => dot,
                        None => continue,
                    };
                    name[..find_dot].to_string()
                }
                Err(_) => continue,
            };

            let file_name_ref = Rc::clone(&rc);
            let path = match file_name_ref.path().to_str() {
                Some(path) => String::from(path),
                None => Err(io::Error::new(io::ErrorKind::Other, "Invalid path"))?,
            };
            temp_hash.insert(file_name, path);

            drop(rc);
        }
        Ok(FolderTables { data: temp_hash })
    }

    /// Given a key (Table name), returns the path to the table
    ///
    /// Else returns None
    pub fn get_path(&self, key: &str) -> Option<String> {
        self.data.get(key).map(|path| path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_tables() {
        let folder = FolderTables::new("./tables").unwrap();

        let expected_tables = vec!["clientes", "ordenes"];

        for table in expected_tables {
            let path = folder.get_path(table);
            assert_eq!(path, Some(format!("./tables/{}.csv", table)));
        }
    }
}

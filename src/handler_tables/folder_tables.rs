use std::{collections::HashMap, fs, rc::Rc};

use crate::errors::tperrors::Tperrors;

/// FolderTables is a struct that contains a HashMap
///
/// With the table name as String, and the path to the table as String
pub struct FolderTables {
    data: HashMap<String, String>,
}

impl FolderTables {
    pub fn new(path_folder: &str) -> Result<FolderTables, Tperrors> {
        let folder = match fs::read_dir(path_folder) {
            Ok(folder) => folder,
            Err(_) => {
                return Err(Tperrors::Table("Folder not found".to_string()));
            }
        };

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
                None => return Err(Tperrors::Table("Invalid path".to_string())),
            };
            temp_hash.insert(file_name, path);
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

    #[test]
    fn test_folder_invalid_folder_throws_err() {
        let folder = FolderTables::new("./invalid_folder");
        assert!(folder.is_err());
    }
}

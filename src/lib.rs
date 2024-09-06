mod conditions;
mod consults;
mod extractor;
mod table;

use conditions::*;
use consults::*;
use extractor::*;
use table::*; // Import the ColumnData type from the conditions module

mod tests {
    use std::collections::HashMap;

    use conditions::Conditions;

    use super::*;

    // using unwrap on test because i know the file exists / its gonna fail.

    #[test]
    fn test_table_new() {
        let table = Table::new(&"./test.csv".to_string()).unwrap();
        assert_eq!(table.get_file_name(), "test".to_string());
    }

    #[test]
    fn test_table_invalid_table() {
        let table = Table::new(&"./invalidtable.csv".to_string());
        assert_eq!(table.is_err(), true);
    }

    #[test]
    fn test_table_invalid_column() {
        let mut table = Table::new(&"./test.csv".to_string()).unwrap();

        // tesis is the invalid columns
        let columns = vec!["Edad".to_string(), "Tesis".to_string()];
        let result = table.execute_select(columns);
        assert_eq!(result.is_err(), true);
    }

}

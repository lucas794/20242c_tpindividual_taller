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

    #[test]
    fn test_extractor_from_consult_extract_columns() {
        let extractor = Extractor::new();
        let consult: &str = "SELECT name, age FROM table;";
        let columns = extractor.extract_columns(&consult.to_string());

        assert_eq!(columns, vec!["name".to_string(), "age".to_string()]);
    }

    #[test]
    fn test_extractor_from_consult_extract_table() {
        let extractor = Extractor::new();

        // here i test the table name
        // should return "table" for both cases
        let consults: Vec<&str> = Vec::from([
            "SELECT name, age FROM table;",
            "SELECT name, age FROM table WHERE name = 'John';",
            "SELECT name, are FROM table ORDER BY name;",
        ]);

        for consult in consults {
            let table = extractor.extract_table(&consult.to_string());
            assert_eq!(table, "table".to_string());
        }
    }

    #[test]
    fn test_select_invalid_query() {
        let select = Select::new();
        let invalid_consults: Vec<&str> = Vec::from([
            "name, age FROM table",
            "SELECT name, age table;",
            "SELECT name, age",
        ]);
        for invalid_query in invalid_consults {
            assert_eq!(select.is_valid_query(&invalid_query.to_string()), false);
        }
    }
    #[test]
    fn test_condition_and() {
        let condition_hash: HashMap<String, Value> = HashMap::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec![
            "name = 'John' AND age = 20",
            "age = 20 AND name = 'John'", // backwards should work too..
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }

    #[test]
    fn test_condition_or() {
        let condition_hash: HashMap<String, Value> = HashMap::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec!["name = 'John'", "age = 20 OR name = 'John'"];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }

    #[test]
    fn test_condition_not() {
        let condition_hash: HashMap<String, Value> = HashMap::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec![
            "NOT name = 'John'", // not
            "NOT age = 20",
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }
}

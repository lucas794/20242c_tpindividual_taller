use std::io::Cursor;

use tp_individual::{
    consults::insert::Insert, errors::tperrors::Tperrors, handler_tables::table::Table,
};
pub mod common;

#[test]
fn integration_insert_query_without_all_columns_used() -> Result<(), Tperrors> {
    let file_name = "insert_query_without_all_columns".to_string();
    let insert = Insert;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());
    let columns_to_insert = vec!["Nombre".to_string(), "Edad".to_string()];
    let values_to_insert = vec!["Juan".to_string(), "20".to_string()];

    match insert.execute_insert_mock(&mut table, columns_to_insert, values_to_insert) {
        Ok(line) => {
            let expected_output = ",Juan,,20,,"; // other commands are NULL.
            assert_eq!(line, expected_output);
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}
#[test]
fn integration_insert_query_with_all_columns_used() -> Result<(), Tperrors> {
    let file_name = "insert_query_with_all_columns".to_string();
    let insert = Insert;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());
    let columns_to_insert = vec![
        "Id".to_string(),
        "Nombre".to_string(),
        "Apellido".to_string(),
        "Edad".to_string(),
        "Correo electronico".to_string(),
        "Profesion".to_string(),
    ];
    let values_to_insert = vec![
        "99".to_string(),
        "Juan".to_string(),
        "Carolo".to_string(),
        "22".to_string(),
        "test@gmail.com".to_string(),
        "maestro".to_string(),
    ];

    match insert.execute_insert_mock(&mut table, columns_to_insert, values_to_insert) {
        Ok(line) => {
            let expected_output = "99,Juan,Carolo,22,test@gmail.com,maestro".to_string();

            assert_eq!(line, expected_output);
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}

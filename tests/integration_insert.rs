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
    // lets create a vector of values to insert
    let values_to_insert = vec![vec!["Juan".to_string(), "20".to_string()]];

    match insert.execute_insert_mock(&mut table, columns_to_insert, values_to_insert) {
        Ok(vec_lines) => {
            let expected_output = ",Juan,,20,,"; // other commands are NULL.
            assert_eq!(vec_lines.len(), 1); // vector of lines should be 1 because we are adding only a value

            for line in vec_lines {
                // for one line
                assert_eq!(line.join(","), expected_output); // join ',' and make the match.
            }
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
    let values_to_insert = vec![vec![
        "99".to_string(),
        "Juan".to_string(),
        "Carolo".to_string(),
        "22".to_string(),
        "test@gmail.com".to_string(),
        "maestro".to_string(),
    ]];

    match insert.execute_insert_mock(&mut table, columns_to_insert, values_to_insert) {
        Ok(vec_lines) => {
            let expected_output = "99,Juan,Carolo,22,test@gmail.com,maestro".to_string();

            assert_eq!(vec_lines.len(), 1); // vector of lines should be 1 because we are adding only one value

            for line in vec_lines {
                assert_eq!(line.join(","), expected_output); // join ',' and make the match.
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}

#[test]
fn integration_insert_query_with_limited_columns_and_multiple_values() -> Result<(), Tperrors> {
    let file_name = "insert_query_with_limited_columns".to_string();
    let insert = Insert;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());
    let columns_to_insert = vec!["Nombre".to_string(), "Edad".to_string()];
    let values_to_insert = vec![
        vec!["Juan".to_string(), "20".to_string()],
        vec!["Pedro".to_string(), "30".to_string()],
        vec!["Maria".to_string(), "40".to_string()],
    ];

    match insert.execute_insert_mock(&mut table, columns_to_insert, values_to_insert) {
        Ok(vec_lines) => {
            let expected_output = vec![",Juan,,20,,", ",Pedro,,30,,", ",Maria,,40,,"];

            assert_eq!(vec_lines.len(), 3); // vector of lines should be 3 because we are adding 3 values

            for (index, line) in vec_lines.iter().enumerate() {
                assert_eq!(line.join(","), expected_output[index]); // join ',' and make the match.
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}

#[test]
fn integration_insert_query_with_all_columns_and_multiple_values() -> Result<(), Tperrors> {
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
        vec![
            "99".to_string(),
            "Juan".to_string(),
            "Carolo".to_string(),
            "22".to_string(),
            "test@gmail.com".to_string(),
            "maestro".to_string(),
        ],
        vec![
            "100".to_string(),
            "Pedro".to_string(),
            "Perez".to_string(),
            "30".to_string(),
            "test@gmail.com".to_string(),
            "electronico".to_string(),
        ],
    ];

    match insert.execute_insert_mock(&mut table, columns_to_insert, values_to_insert) {
        Ok(vec_lines) => {
            let vec_expected_string_output = vec![
                "99,Juan,Carolo,22,test@gmail.com,maestro",
                "100,Pedro,Perez,30,test@gmail.com,electronico",
            ];

            assert_eq!(vec_lines.len(), 2); // vector of lines should be 2 because we are adding 2 values

            for (i, line) in vec_lines.iter().enumerate() {
                assert_eq!(line.join(","), vec_expected_string_output[i]); // join ',' and make the match with the [i] output
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}

use std::io::Cursor;

use tp_individual::{
    consults::select::Select, errors::tperrors::Tperrors, handler_tables::table::Table,
};

pub mod common;

#[test]
fn integration_simple_select_query() -> Result<(), Tperrors> {
    let file_name = String::from("query_select_simple_query");
    let select = Select;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["Nombre".to_string(), "Edad".to_string()]);

    let condition = Some("Edad >=33");
    let sort_method = None;

    match select.execute_select_mock(&mut table, columns, condition, sort_method) {
        Ok(vector_of_lines) => {
            let expected_output: Vec<Vec<&str>> = vec![
                vec!["Nombre", "Edad"],
                vec!["Carlos", "45"],
                vec!["Ana", "36"],
                vec!["Laura", "41"],
                vec!["Pedro", "33"],
                vec!["Diego", "39"],
            ];

            for (i, row) in vector_of_lines.iter().enumerate() {
                let expected_row = &expected_output[i];

                for (j, cell) in row.iter().enumerate() {
                    assert_eq!(cell, &expected_row[j]);
                }
            }
        }
        Err(e) => {
            // this test has failed due a temp file wasnt abled to be generated.
            // this means it wasnt able to generate a temp file for any reason.
            return Err(e);
        }
    }
    Ok(())
}

#[test]
fn integration_advanced_select_query() -> Result<(), Tperrors> {
    // "SELECT Nombre, Edad FROM database WHERE (Nombre = Luis OR Edad > 15) AND NOT Nombre = Paula;\"

    let file_name = String::from("query_select_advanced_query");
    let select = Select;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["Nombre".to_string(), "Edad".to_string()]);

    let condition = Some("(Nombre = Luis OR Edad>15) AND NOT Nombre = Paula");
    let sort_method = None;

    match select.execute_select_mock(&mut table, columns, condition, sort_method) {
        Ok(vector_of_lines) => {
            let expected_output = vec![
                vec!["Nombre", "Edad"],
                vec!["Juan", "32"],
                vec!["Maria", "28"],
                vec!["Carlos", "45"],
                vec!["Ana", "36"],
                vec!["Luis", "29"],
                vec!["Laura", "41"],
                vec!["Pedro", "33"],
                vec!["Lucía", "26"],
                vec!["Diego", "39"],
            ];

            for (i, row) in vector_of_lines.iter().enumerate() {
                let expected_row = &expected_output[i];

                for (j, cell) in row.iter().enumerate() {
                    assert_eq!(cell, &expected_row[j]);
                }
            }
        }
        Err(e) => {
            // this test has failed due a temp file wasnt abled to be generated.
            // this means it wasnt able to generate a temp file for any reason.
            return Err(e);
        }
    }
    Ok(())
}

#[test]
fn integration_advanced_test_constant() -> Result<(), Tperrors> {
    // \"SELECT * FROM database WHERE 1=1;\"

    let file_name = String::from("query_select_advanced_test_constant");
    let select = Select;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["*".to_string()]);

    let condition = Some("1=1");
    let sort_method = None;

    match select.execute_select_mock(&mut table, columns, condition, sort_method) {
        Ok(vector_of_lines) => {
            for (vec_output, line_from_csv_mock) in vector_of_lines
                .iter()
                .zip(common::return_iterator_from_mock_csv_file())
            {
                let line_output = vec_output.join(",");
                assert_eq!(line_output, line_from_csv_mock);
            }
        }
        Err(e) => {
            // this test has failed due a temp file wasnt abled to be generated.
            // this means it wasnt able to generate a temp file for any reason.
            return Err(e);
        }
    }
    Ok(())
}

#[test]
fn integration_advanced_select_query_with_columns_from_table_as_conditions() -> Result<(), Tperrors>
{
    // \"SELECT * FROM database WHERE Id>Edad;\"

    let file_name =
        String::from("query_select_advanced_query_with_columns_from_table_as_conditions");
    let select = Select;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["*".to_string()]);

    let condition = Some("Id>Edad");
    let sort_method = None;

    match select.execute_select_mock(&mut table, columns, condition, sort_method) {
        Ok(vector_of_lines) => {
            // Making a SELECT * FROM database WHERE Id>Edad; should return only the headers
            let expected_header_as_vec = [
                "Id",
                "Nombre",
                "Apellido",
                "Edad",
                "Correo electronico",
                "Profesion",
            ];

            for (_i, row) in vector_of_lines.iter().enumerate() {
                let expected_row = &expected_header_as_vec;

                for (j, cell) in row.iter().enumerate() {
                    assert_eq!(cell, &expected_row[j]);
                }
            }
        }
        Err(e) => {
            // this test has failed due a temp file wasnt abled to be generated.
            // this means it wasnt able to generate a temp file for any reason.
            return Err(e);
        }
    }
    Ok(())
}

#[test]
fn integration_select_advanced_query_with_nested_conditions_and_spaced_conditions(
) -> Result<(), Tperrors> {
    // \"SELECT * FROM database WHERE Edad <30 AND (Nombre!= 'Luis' AND Nombre !='Maria');\"

    let file_name =
        String::from("query_select_advanced_query_with_nested_conditions_and_spaced_conditions");
    let select = Select;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["*".to_string()]);

    // Check out i'm mixingg attached conditions with spaced conditions and it still works.

    let condition = Some("Edad<30 AND (Nombre != 'Luis' AND Nombre !='Maria')");
    let sort_method = None;

    match select.execute_select_mock(&mut table, columns, condition, sort_method) {
        Ok(vector_of_lines) => {
            // we need to match 8,Lucía,Ramos,26,lramos@gmail.com,psicóloga
            const INDEX_HEADER: usize = 0;
            const INDEX_ROW_LUCIA: usize = 8;
            let expected_results: Vec<String> = vec![
                common::return_iterator_from_mock_csv_file()
                    .nth(INDEX_HEADER)
                    .unwrap(),
                common::return_iterator_from_mock_csv_file()
                    .nth(INDEX_ROW_LUCIA)
                    .unwrap(),
            ];

            for (vec_output, line_from_csv_mock) in
                vector_of_lines.iter().zip(expected_results.iter())
            {
                let line_output = vec_output.join(",");
                assert_eq!(line_output, line_from_csv_mock.to_string());
            }
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

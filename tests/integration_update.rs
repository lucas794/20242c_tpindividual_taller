use std::io::{BufRead, Cursor};

use tp_individual::{
    consults::update::Update, errors::tperrors::Tperrors, handler_tables::table::Table,
};

pub mod common;

#[test]
fn integration_update_simple_query() -> Result<(), Tperrors> {
    let file_name = String::from("query_update_simple_query");
    let update = Update;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["Nombre".to_string(), "Edad".to_string()]);
    let values: Vec<String> = Vec::from(vec!["TEST".to_string(), "45".to_string()]);

    let condition = Some("Edad =31");

    // this will replace the last entry of the mocked file
    match update.execute_update_mock(&mut table, columns, values, condition) {
        Ok(buf_reader) => {
            let last_line = buf_reader.lines().last().unwrap().unwrap();
            let expected_output = "10,TEST,Hernández,45,phernandez@gmail.com,publicista";
            assert_eq!(last_line, expected_output);
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
fn integration_update_all_values_query() -> Result<(), Tperrors> {
    let file_name = String::from("query_update_all_values_query");
    let update = Update;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let columns: Vec<String> = Vec::from(vec!["Nombre".to_string(), "Edad".to_string()]);
    let values: Vec<String> = Vec::from(vec!["TEST".to_string(), "45".to_string()]);

    let condition = None;

    // this will replace all entry of the mocked file :D
    match update.execute_update_mock(&mut table, columns, values, condition) {
        Ok(buf_reader) => {
            // headers should match at start, so now we will check the next lines
            const INDEX_COLUMN_NAME: usize = 1;
            const INDEX_COLUMN_AGE: usize = 3;
            for line in buf_reader.lines().skip(1) {
                let line = line.unwrap();
                let columns = line.split(",").collect::<Vec<&str>>();
                assert_eq!(columns[INDEX_COLUMN_NAME], "TEST");
                assert_eq!(columns[INDEX_COLUMN_AGE], "45");
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

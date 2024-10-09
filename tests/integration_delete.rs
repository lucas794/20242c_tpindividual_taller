use std::io::{BufRead, Cursor};

use tp_individual::{
    consults::delete::Delete, errors::tperrors::Tperrors, handler_tables::table::Table,
};

pub mod common;

#[test]
fn integration_delete_paula_from_database() -> Result<(), Tperrors> {
    // paula is the last entry on our mock file.
    let file_name = String::from("delete_query_deletion_paula");
    let delete = Delete;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());
    let condition = Some("Nombre=Paula");

    match delete.execute_delete_mock(&mut table, condition) {
        Ok(mocked_file) => {
            let last_line = mocked_file.lines().last().unwrap().unwrap();

            let expected_output = "9,Diego,Navarro,39,dnavarro@gmail.com,empresario";

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
fn integration_delete_whole_database_query() -> Result<(), Tperrors> {
    // paula is the last entry on our mock file.
    let file_name = String::from("query_delete_whole_database");
    let delete = Delete;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());

    let condition = None;

    match delete.execute_delete_mock(&mut table, condition) {
        Ok(mocked_file) => {
            let expected_output = "Id,Nombre,Apellido,Edad,Correo electronico,Profesion";

            for line in mocked_file.lines() {
                // we do this to read the whole file and find the matches, if any.
                let line = line.unwrap();
                assert_eq!(line, expected_output);
            }
            // so we iterated for the whole file, and we matched the expected output, so we are good.
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

#[test]
fn integration_delete_expanded_query_with_column_as_condition() -> Result<(), Tperrors> {
    // We are gonna try to simulate a
    // DELETE FROM clientes WHERE (Id>=2 AND Edad>=30 AND 1=1);"
    // this is gonna return rows with id 1,2, 5, 8 according sandboxSQL.

    let file_name = String::from("delete_expanded_query_with_column_as_condition");
    let delete = Delete;
    let mut table = Table::<Cursor<&[u8]>>::mock(file_name, common::csv_data_as_bytes());
    let condition = Some("Id>=2 AND Edad>=30 AND 1=1");

    match delete.execute_delete_mock(&mut table, condition) {
        Ok(mocked_file) => {
            let expected_output_vectors = vec![
                "Id,Nombre,Apellido,Edad,Correo electronico,Profesion", // ofc we are gonna have the header.
                "1,Juan,Perez,32,jperez@gmail.com,medico",
                "2,Maria,Gomez,28,mgomez@gmail.com,abogado",
                "5,Luis,Martínez,29,lmartinez@gmail.com,profesor",
                "8,Lucía,Ramos,26,lramos@gmail.com,psicóloga",
            ];

            for (i, line_read) in mocked_file.lines().enumerate() {
                let line = line_read.unwrap();
                assert_eq!(line, expected_output_vectors[i]);
            }
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

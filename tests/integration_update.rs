use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::Command,
};

pub mod common;
#[test]
fn integration_update_simple_query() {
    // create a new file;
    let route_file = format!("./tests/update_query_simple_query.csv");
    common::setup(&route_file);

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    let argument = format!(
        "cargo run -- ./tests \"UPDATE {} SET Nombre = 'TEST', Edad = 45 WHERE Edad = 31;\"",
        table_name
    );

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    // lets read the last line of the file
    let reader = BufReader::new(File::open(&route_file).unwrap());
    let _ = std::fs::remove_file(&route_file).unwrap();

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "10,TEST,Hernández,45,phernandez@gmail.com,publicista";

    assert_eq!(last_line, expected_output);
}

#[test]
fn integration_update_all_values_query() {
    // create a new file;
    let route_file = format!("./tests/update_query_update_all.csv");
    common::setup(&route_file);

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    let argument = format!(
        "cargo run -- ./tests \"UPDATE {} SET Nombre = Lucas, Edad = 32;\"",
        table_name
    );

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();
    // lets read the last line of the file
    let reader = BufReader::new(File::open(&route_file).unwrap());
    let _ = std::fs::remove_file(&route_file).unwrap();

    const COLUMN_NAME_INDEX: usize = 1;
    const COLUMN_AGE_INDEX: usize = 3;

    for line in reader.lines().skip(1) {
        // skip headers
        let line = line.unwrap();
        let name_column = line.split(",").nth(COLUMN_NAME_INDEX).unwrap();
        let age_column = line.split(",").nth(COLUMN_AGE_INDEX).unwrap();

        // all name, age columns should match lucas,32
        assert_eq!((name_column, age_column), ("Lucas", "32"));
    }
}

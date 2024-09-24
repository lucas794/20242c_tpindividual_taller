use std::{
    fs::{self, File},
    io::{self},
    process,
};

use io::{BufRead, BufReader};
use process::Command;

#[test]
fn integration_simple_select_query() {
    let route_file = format!("./tests/select_query_{}.csv", std::process::id());
    let argument = format!("cargo run -- ./tests/data \"SELECT Nombre, Edad FROM database WHERE Edad >= 33;\" > {}", route_file);
    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    let file = File::open(&route_file).unwrap();

    let expected_output: Vec<Vec<&str>> = vec![
        vec!["Nombre", "Edad"],
        vec!["Carlos", "45"],
        vec!["Ana", "36"],
        vec!["Laura", "41"],
        vec!["Pedro", "33"],
        vec!["Diego", "39"],
    ];

    // read  the file and compare it with the expected output
    let reader = BufReader::new(file);

    let mut output: Vec<Vec<String>> = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let line_to_vec: Vec<String> = line.split(",").map(|s| s.to_string()).collect();

        output.push(line_to_vec);
    }

    for (i, row) in output.iter().enumerate() {
        let expected_row = &expected_output[i];

        for (j, cell) in row.iter().enumerate() {
            assert_eq!(cell, &expected_row[j]);
        }
    }

    // lets remove the recent created file
    let _ = std::fs::remove_file(&route_file);
}

#[test]
fn integration_advanced_select_query() {
    let route_file = format!("./tests/select_advanced_query_{}.csv", std::process::id());
    let argument = format!("cargo run -- ./tests/data \"SELECT Nombre, Edad FROM database WHERE (Nombre = Luis OR Edad > 15) AND NOT Nombre = Paula;\" > {}", route_file);
    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    let file = File::open(&route_file).unwrap();

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

    // read  the file and compare it with the expected output
    let reader = BufReader::new(file);

    let mut output: Vec<Vec<String>> = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let line_to_vec: Vec<String> = line.split(",").map(|s| s.to_string()).collect();

        output.push(line_to_vec);
    }

    for (i, row) in output.iter().enumerate() {
        let expected_row = &expected_output[i];

        for (j, cell) in row.iter().enumerate() {
            assert_eq!(cell, &expected_row[j]);
        }
    }

    // lets remove the recent created file
    let _ = std::fs::remove_file(&route_file);
}
#[test]
fn integration_insert_query() {
    // create a new file;
    let route_file = format!("./tests/insert_query_{}.csv", std::process::id());

    // create a file
    let _ = File::create(&route_file).unwrap();

    // lets clone the file
    fs::copy("./tests/data/database.csv", &route_file).unwrap();

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    let argument = format!(
        "cargo run -- ./tests \"INSERT INTO {} (Nombre, Edad) VALUES ('Juan', 20);\"",
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

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "Juan,,20,,"; // other commands are NULL.

    let _ = std::fs::remove_file(&route_file);

    assert_eq!(last_line, expected_output);
}

#[test]
fn integration_update_query() {
    // create a new file;
    let route_file = format!("./tests/update_query_{}.csv", std::process::id());

    // create a file
    let _ = File::create(&route_file).unwrap();

    // lets clone the file
    fs::copy("./tests/data/database.csv", &route_file).unwrap();

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

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "TEST,Hernández,45,phernandez@gmail.com,publicista";

    let _ = std::fs::remove_file(&route_file).unwrap();

    assert_eq!(last_line, expected_output);
}

#[test]
fn integration_delete_query() {
    // create a new file;
    let route_file = format!("./tests/delete_query_{}.csv", std::process::id());

    // create a file
    let _ = File::create(&route_file).unwrap();

    // lets clone the file
    fs::copy("./tests/data/database.csv", &route_file).unwrap();

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    let argument = format!("cargo run -- ./tests \"DELETE FROM {};\"", table_name);

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    // we are basically deleting the whole database, ONLY the header remains..

    let reader = BufReader::new(File::open(&route_file).unwrap());

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "Nombre,Apellido,Edad,Correo electronico,Profesion";

    assert_eq!(last_line, expected_output);

    let _ = std::fs::remove_file(&route_file).unwrap();

    assert_eq!(last_line, expected_output);
}

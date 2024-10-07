use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::Command,
};

pub mod common;

#[test]
fn integration_simple_select_query() {
    let route_file = format!("./tests/select_query_simple.csv");
    let argument = format!(
        "cargo run -- ./tests/data \"SELECT Nombre, Edad FROM database WHERE Edad >= 33;\" > {}",
        route_file
    );
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

    let _ = std::fs::remove_file(&route_file);

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
}

#[test]
fn integration_advanced_select_query() -> Result<(), std::io::Error> {
    let route_file = String::from("./tests/select_advanced_query_with_parenthesis.csv");

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

    let _ = std::fs::remove_file(&route_file);

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

    Ok(())
}

#[test]
fn integration_advance_test_constant() -> Result<(), std::io::Error> {
    // Lets simualte a query with condition as 1=1, so we return everything.

    let route_file =
        String::from("./tests/select_advanced_query_using_constants_as_conditions.csv");

    let argument = format!(
        "cargo run -- ./tests/data \"SELECT * FROM database WHERE 1=1;\" > {}",
        route_file
    );

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    let file = File::open(&route_file).unwrap();

    // read  the file and compare it with the expected output
    let reader = BufReader::new(file);

    let _ = std::fs::remove_file(&route_file);

    for (line_reader, line_from_csv_mock) in reader
        .lines()
        .zip(common::return_iterator_from_mock_csv_file())
    {
        let line = line_reader.unwrap();
        assert_eq!(line, line_from_csv_mock);
    }

    Ok(())
}

#[test]
fn integration_advanced_select_query_with_columns_from_table_as_conditions(
) -> Result<(), std::io::Error> {
    // Lets simualte a query with condition as 1=1, so we return everything.
    /*let start = SystemTime::now();
    let since_the_epoch = match start.duration_since(UNIX_EPOCH) {
        Ok(time) => time,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error getting time",
            ));
        }
    };

    let route_file = format!(
        "./tests/select_advanced_query_{}.csv",
        since_the_epoch.as_nanos() + (std::process::id() as u128)
    );*/

    let route_file =
        String::from("./tests/select_advanced_query_with_columns_from_table_as_conditions.csv");
    let argument = format!(
        "cargo run -- ./tests/data \"SELECT * FROM database WHERE Id>Edad;\" > {}",
        route_file
    );

    // Making a SELECT * FROM database WHERE Id>Edad; should return only the headers

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    let file = File::open(&route_file).unwrap();

    // read  the file and compare it with the expected output
    let reader = BufReader::new(file);
    let _ = std::fs::remove_file(&route_file); // we delete the temporal file

    for (line_reader, line_from_csv_mock) in reader
        .lines()
        .zip(common::return_iterator_from_mock_csv_file())
    {
        let line = line_reader.unwrap();
        assert_eq!(line, line_from_csv_mock);
    }

    Ok(())
}

#[test]
fn integration_select_advanced_query_with_nested_conditions_and_spaced_conditions(
) -> Result<(), std::io::Error> {
    let route_file =
        String::from("./tests/integration_select_advanced_query_with_nested_conditions_and_spaced_conditions.csv");
    let argument = format!(
        "cargo run -- ./tests/data \"SELECT * FROM database WHERE Edad <30 AND (Nombre!= 'Luis' AND Nombre !='Maria');\" > {}",
        route_file
    );

    // we mix spaces and no spaces in the conditions

    // Making a SELECT * FROM database WHERE Id>Edad; should return only the headers

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    let file = File::open(&route_file).unwrap();

    // read  the file and compare it with the expected output
    let reader = BufReader::new(file);
    let _ = std::fs::remove_file(&route_file);

    // we need to match 8,Lucía,Ramos,26,lramos@gmail.com,psicóloga
    // that is located at line 9 of the csv mocked.
    // and also the HEADER of the csv file

    const ROW_HEADER_INDEX: usize = 0;
    const ROW_LUCIA_INDEX: usize = 8;

    let expected_results: Vec<String> = vec![
        common::return_iterator_from_mock_csv_file()
            .nth(ROW_HEADER_INDEX)
            .unwrap(),
        common::return_iterator_from_mock_csv_file()
            .nth(ROW_LUCIA_INDEX)
            .unwrap(),
    ];

    for (line_reader, line_from_csv_mock) in reader.lines().zip(expected_results.iter()) {
        let line = line_reader.unwrap();
        assert_eq!(line, line_from_csv_mock.to_string());
    }

    Ok(())
}

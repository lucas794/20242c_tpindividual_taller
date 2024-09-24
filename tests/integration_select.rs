use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::Command,
};

#[test]
fn integration_simple_select_query() {
    let route_file = format!("./tests/select_query_{}.csv", std::process::id());
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
    std::thread::sleep(std::time::Duration::from_millis(30));
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
    std::thread::sleep(std::time::Duration::from_millis(30));
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

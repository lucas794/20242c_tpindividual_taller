use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    process::Command,
};

#[test]
fn integration_update_simple_query() {
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
    std::thread::sleep(std::time::Duration::from_millis(30));
    // lets read the last line of the file
    let reader = BufReader::new(File::open(&route_file).unwrap());

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "TEST,Hernández,45,phernandez@gmail.com,publicista";

    let _ = std::fs::remove_file(&route_file).unwrap();

    assert_eq!(last_line, expected_output);
}

#[test]
fn integration_update_all_values_query() {
    // create a new file;
    let route_file = format!("./tests/update_query_{}.csv", std::process::id() + 124);

    // create a file
    let _ = File::create(&route_file).unwrap();

    // lets clone the file
    fs::copy("./tests/data/database.csv", &route_file).unwrap();

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    println!("table_name: {}", table_name);

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

    for line in reader.lines().skip(1) {
        // skip headers
        let line = line.unwrap();
        let name_column = line.split(",").nth(0).unwrap();
        let age_column = line.split(",").nth(2).unwrap();

        assert_eq!((name_column, age_column), ("Lucas", "32"));
    }

    let _ = std::fs::remove_file(&route_file).unwrap();
}

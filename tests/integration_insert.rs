use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    process::Command,
};

#[test]
fn integration_insert_query_without_all_columns_used() {
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
    std::thread::sleep(std::time::Duration::from_millis(30));
    // lets read the last line of the file
    let reader = BufReader::new(File::open(&route_file).unwrap());

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "Juan,,20,,"; // other commands are NULL.

    let _ = std::fs::remove_file(&route_file);

    assert_eq!(last_line, expected_output);
}

#[test]
fn integration_insert_query_with_all_columns_used() {
    // create a new file;
    let route_file = format!("./tests/insert_query_{}.csv", std::process::id() + 1);

    // create a file
    let _ = File::create(&route_file).unwrap();

    // lets clone the file
    fs::copy("./tests/data/database.csv", &route_file).unwrap();

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    let argument = format!(
        "cargo run -- ./tests \"INSERT INTO {} (Nombre, Apellido, Edad, \'Correo electronico\', Profesion) VALUES ('Juan', 'Carolo', '22', 'test@gmail.com', 'maestro');\"",
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

    let expected_output = "Juan,Carolo,22,test@gmail.com,maestro"; // other commands are NULL.

    let _ = std::fs::remove_file(&route_file);

    assert_eq!(last_line, expected_output);
}

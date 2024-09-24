use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    process::Command,
};

#[test]
fn integration_delete_paula_from_database() {
    // paula is at the last position of the database
    // create a new file;
    let route_file = format!("./tests/delete_query_{}.csv", std::process::id() + 2424);

    // create a file
    let _ = File::create(&route_file).unwrap();

    // lets clone the file
    fs::copy("./tests/data/database.csv", &route_file).unwrap();

    let table_name_start = route_file.rfind("/").unwrap() + 1;
    let table_name_end = route_file.rfind(".").unwrap();
    let table_name = &route_file[table_name_start..table_name_end];

    let argument = format!(
        "cargo run -- ./tests \"DELETE FROM {} WHERE Nombre = Paula;\"",
        table_name
    );

    let mut command = Command::new("sh") // Use "cmd" for Windows
        .arg("-c") // Execute a shell command
        .arg(argument)
        .spawn()
        .unwrap();

    command.wait().unwrap();

    // we are basically deleting the whole database, ONLY the header remains..

    let reader = BufReader::new(File::open(&route_file).unwrap());

    let last_line = reader.lines().last().unwrap().unwrap();

    let expected_output = "Diego,Navarro,39,dnavarro@gmail.com,empresario";

    assert_eq!(last_line, expected_output);

    let _ = std::fs::remove_file(&route_file).unwrap();

    assert_eq!(last_line, expected_output);
}

#[test]
fn integration_delete_whole_database_query() {
    // create a new file;
    let route_file = format!("./tests/delete_query_{}.csv", std::process::id() + 5252);

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

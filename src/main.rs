mod consults;
mod table;

use consults::*;
use table::Table;

fn main() {
    // lets read the args
    let args: Vec<String> = std::env::args().collect();

    if !valid_number_of_args(&args.len()) {
        return; // we stop the program, invalid number of arguments
    }

    // Now, we have the arguments...
    let file = &args[1];
    let consult = &args[2].trim().to_string();

    println!("File: {}", file);
    println!("Consult: [{}]", consult);

    let table = Table::new(file);
    println!("Table name: {}", table.get_file_name());

    let splitted_consult = consult.split(" ").collect::<Vec<&str>>();
    let command = splitted_consult[0];

    match command {
        "select" | "SELECT" => {
            let command = Select::new();

            if !command.is_valid_query(&consult) {
                println!("[INVALID_SYNTAX]: Invalid select query");
                return;
            }

            command.get_query_parts(&consult);
        }
        "insert" | "INSERT" => {
            println!("Insert command");
        }
        "update" | "UPDATE" => {
            println!("Update command");
        }
        "delete" | "DELETE" => {
            println!("Delete command");
        }
        _ => {
            println!("[INVALID_SYNTAX]: Invalid command");
            return;
        }
    }
}

fn valid_number_of_args(args: &usize) -> bool {
    if *args <= 2 {
        println!("Invalid number of arguments!");
        return false;
    }
    true
}

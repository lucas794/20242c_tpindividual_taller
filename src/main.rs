mod consults;
mod extractor;
mod table;

use consults::*;
use extractor::*;
use table::*;

fn main() {
    // lets read the args
    let args: Vec<String> = std::env::args().collect();

    if !valid_number_of_args(&args.len()) {
        return; // we stop the program, invalid number of arguments
    }

    // Now, we have the arguments...
    let file = &args[1];
    let consult = &args[2].trim().to_string();

    let opening_table = Table::new(file);

    let mut table = match opening_table {
        Ok(table) => table,
        Err(_) => {
            println!("[INVALID_TABLE]: The selected table is invalid");
            return;
        }
    };

    let splitted_consult = consult.split(" ").collect::<Vec<&str>>();
    let command = splitted_consult[0];

    let extractor = Extractor::new();

    match command {
        "SELECT" => {
            let command = Select::new();

            if !command.is_valid_query(&consult) {
                println!("[INVALID_SYNTAX]: Invalid select query");
                return;
            }

            let table_name = extractor.extract_table(&consult);

            if table_name != table.get_file_name() {
                println!("[INVALID_TABLE]: Table name does not match");
                return;
            }

            let conditions = extractor.extract_conditions(&consult);
            println!("Conditions: {:?}", conditions);

            let columns = extractor.extract_columns(&consult);

            let _ = table.execute_select(columns);
        }
        "INSERT" => {
            println!("Insert command");
        }
        "UPDATE" => {
            println!("Update command");
        }
        "DELETE" => {
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

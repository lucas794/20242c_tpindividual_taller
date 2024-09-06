mod conditions;
mod consults;
mod errors;
mod extractor;
mod table;

use std::io::{self, Write};

use consults::Select;
use errors::TPErrors;
use extractor::Extractor;
use table::Table;

fn main() {
    // lets read the args
    let args: Vec<String> = std::env::args().collect();

    if let Err(e) = run(args) {
        eprintln!("{}", e);
    }
}

/// check if the number of arguments is valid
/// returns false if the number is equal or lower than 2
fn valid_number_of_args(args: &usize) -> bool {
    if *args <= 2 {
        return false;
    }
    true
}

/// Executes the main logical problem of the program
fn run(args: Vec<String>) -> Result<(), errors::TPErrors<'static>> {
    if !valid_number_of_args(&args.len()) {
        // we stop the program, invalid number of arguments
        return Err(TPErrors::InvalidGeneric("Invalid number of arguments"));
    }

    // Now, we have the arguments...
    let file = &args[1];
    let consult = &args[2].trim();

    let opening_table = Table::new(file);

    let mut table = match opening_table {
        Ok(table) => table,
        Err(_) => {
            return Err(TPErrors::InvalidTable(
                "Error opening the table - The selected table is invalid",
            ));
        }
    };

    let splitted_consult = consult.split(" ").collect::<Vec<&str>>();
    let command = splitted_consult[0];

    let extractor = Extractor::new();

    match command {
        "SELECT" => {
            let command = Select::new();

            if !command.is_valid_query(consult) {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid select query (Missing either SELECT , FROM or ;)",
                ));
            }

            match extractor.extract_table(&consult) {
                Ok(table_name_from_query) => {
                    let table_name = match table.get_file_name() {
                        Ok(table_name) => table_name,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    if table_name_from_query != table_name {
                        return Err(TPErrors::InvalidTable("Invalid table selected"));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            };

            let columns = match extractor.extract_columns(consult) {
                Ok(columns) => columns,
                Err(e) => {
                    return Err(e);
                }
            };

            // to use later.
            let conditions = extractor.extract_as_str_conditions(&consult);

            let sorting_vector = match extractor.extract_orderby_as_str(&consult) {
                Some(sorting) => {
                    println!("Sorting: [{}]", sorting);
                    let vec_sort = extractor.parser_order_by_str(sorting);
                    Some(vec_sort)
                }
                None => None,
            };

            let csv_data = table.execute_select(columns, conditions, None);

            match csv_data {
                Ok(data) => {
                    // lets write stdout
                    let stdout = io::stdout();

                    let mut handle = io::BufWriter::new(stdout.lock());

                    for line in data {
                        let temp_line = line.join(",");
                        handle.write(temp_line.as_bytes()).unwrap();
                        handle.write(b"\n").unwrap();
                    }
                }
                Err(_) => {
                    return Err(TPErrors::InvalidSyntax("Invalid columns inside the query"));
                }
            }
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
            return Err(TPErrors::InvalidSyntax("Invalid command"));
        }
    }
    Ok(())
}

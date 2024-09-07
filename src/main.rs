mod conditions;
mod consults;
mod errors;
mod extractor;
mod table;

use consults::{Insert, Select};
use errors::TPErrors;
use extractor::{Extractor, SQLCommand};
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
            let select = Select::new();

            if !select.is_valid_query(consult) {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid select query (Missing either SELECT , FROM or ;)",
                ));
            }

            // checking if its a valid table
            match extractor.extract_table(&consult, SQLCommand::SELECT) {
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

            // lets get the columns selected from the query
            let columns = match extractor.extract_columns_for_select(consult) {
                Ok(columns) => columns,
                Err(e) => {
                    return Err(e);
                }
            };

            // Conditions of the query (if they exists)
            let conditions = extractor.extract_as_str_conditions(&consult);

            // Sorting method (if existst)
            let sorting_vector = match extractor.extract_orderby_as_str(&consult) {
                Some(sorting) => {
                    let vec_sort = extractor.parser_orderby_from_str_to_vec(sorting);
                    Some(vec_sort)
                }
                None => None,
            };

            // lets execute the query
            let result = select.execute_select(&mut table, columns, conditions, sorting_vector);

            if result.is_err() {
                return Err(TPErrors::InvalidSyntax("Invalid columns inside the query"));
            }

            return Ok(());
        }
        "INSERT" => {
            let insert = Insert::new();

            if !insert.is_valid_query(consult) {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid insert query (Missing either INSERT INTO, VALUES or ;)",
                ));
            }

            let table_from_query = match extractor.extract_table(&consult, SQLCommand::INSERT) {
                Ok(table_as_string) => table_as_string,
                Err(e) => {
                    return Err(e);
                }
            };

            match table.get_file_name() {
                Ok(table_name) => {
                    if table_from_query != table_name {
                        return Err(TPErrors::InvalidTable("Invalid table selected"));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            };

            let (columns, values) = match extractor.extract_columns_and_values_for_insert(consult) {
                Ok((columns, values)) => (columns, values),
                Err(e) => {
                    return Err(e);
                }
            };

            let result = insert.execute_insert(&mut table, columns, values);

            if result.is_err() {
                return Err(TPErrors::InvalidSyntax("Invalid columns inside the query"));
            }

            return Ok(());
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

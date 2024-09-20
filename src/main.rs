use tp_individual::{
    consults::{delete::Delete, insert::Insert, select::Select, update::Update},
    errors::tperrors::Tperrors,
    extractors::{extractor::Extractor, sqlcommand::SQLCommand},
    table::Table,
};

fn main() {
    // lets read the args
    let args: Vec<String> = std::env::args().collect();

    if let Err(e) = run(args) {
        eprintln!("{}", e);
    }
}

/// check if the number of arguments is valid
///
/// returns false if the number is equal or lower than 2
fn valid_number_of_args(args: &usize) -> bool {
    if *args <= 2 {
        return false;
    }
    true
}

/// This function checks if the table selected in the query is the same as the table that is being used
fn check_valid_table(
    extractor: &Extractor,
    table: &Table,
    consult: &str,
    method: SQLCommand,
) -> Result<(), Tperrors<'static>> {
    match extractor.extract_table(consult, method) {
        Ok(table_name_from_query) => {
            let table_name = match table.get_file_name() {
                Ok(table_name) => table_name,
                Err(e) => {
                    return Err(e);
                }
            };

            if table_name_from_query != table_name {
                return Err(Tperrors::Table("Invalid table selected"));
            }
        }
        Err(e) => {
            return Err(e);
        }
    };
    Ok(())
}
/// Executes the main logical problem of the program
fn run(args: Vec<String>) -> Result<(), Tperrors<'static>> {
    if !valid_number_of_args(&args.len()) {
        // we stop the program, invalid number of arguments
        return Err(Tperrors::Generic("Invalid number of arguments"));
    }

    // Now, we have the arguments...
    let file = &args[1];
    let consult = &args[2].trim();

    let opening_table = Table::new(file);

    let mut table = match opening_table {
        Ok(table) => table,
        Err(_) => {
            return Err(Tperrors::Table(
                "Error opening the table - The selected table is invalid",
            ));
        }
    };

    let splitted_consult = consult.split(" ").collect::<Vec<&str>>();
    let command = splitted_consult[0];

    let extractor = Extractor;

    match command {
        "SELECT" => {
            let select = Select;

            if !select.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid select query (Missing either SELECT , FROM or ;)",
                ));
            }

            // checking if its a valid table
            match check_valid_table(&extractor, &table, consult, SQLCommand::Select) {
                Ok(_) => {}
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
            let conditions = extractor.extract_as_str_conditions(consult);

            // Sorting method (if existst)
            let sorting_vector = match extractor.extract_orderby_as_str(consult) {
                Some(sorting) => {
                    let vec_sort = extractor.parser_orderby_from_str_to_vec(sorting);
                    Some(vec_sort)
                }
                None => None,
            };

            // lets execute the query
            let result = select.execute_select(&mut table, columns, conditions, sorting_vector);

            if result.is_err() {
                return Err(Tperrors::Syntax("Invalid columns inside the query"));
            }

            return Ok(());
        }
        "INSERT" => {
            let insert = Insert;

            if !insert.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid insert query (Missing either INSERT INTO, VALUES or ;)",
                ));
            }

            match check_valid_table(&extractor, &table, consult, SQLCommand::Insert) {
                Ok(_) => {}
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
                return Err(Tperrors::Syntax("Invalid columns inside the query"));
            }

            return Ok(());
        }
        "UPDATE" => {
            let update = Update;

            if !update.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid update query (Missing either UPDATE, SET, WHERE or ;)",
                ));
            }

            match check_valid_table(&extractor, &table, consult, SQLCommand::Update) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };

            let (columns, values) = match extractor.extract_columns_and_values_for_update(consult) {
                Ok((columns, values)) => (columns, values),
                Err(e) => {
                    return Err(e);
                }
            };

            let conditions = extractor.extract_as_str_conditions(consult);

            let result = update.execute_update(&mut table, columns, values, conditions);

            if result.is_err() {
                return Err(Tperrors::Syntax("Invalid columns inside the query"));
            }
        }
        "DELETE" => {
            let delete = Delete;

            if !delete.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid delete query (Missing either DELETE, FROM or ;)",
                ));
            }

            // checking if its a valid table
            match check_valid_table(&extractor, &table, consult, SQLCommand::Delete) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };

            let conditions = extractor.extract_as_str_conditions(consult);

            let result = delete.execute_delete(&mut table, conditions);

            if result.is_err() {
                return Err(Tperrors::Generic("Something happened with the deletion"));
            }
        }
        _ => {
            return Err(Tperrors::Syntax("Invalid command"));
        }
    }
    Ok(())
}

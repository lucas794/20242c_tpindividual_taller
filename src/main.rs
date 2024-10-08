use std::fs::File;

use tp_individual::{
    consults::{delete::Delete, insert::Insert, select::Select, update::Update},
    errors::tperrors::Tperrors,
    extractors::{extractor::Extractor, sqlcommand::SQLCommand},
    handler_tables::folder_tables::FolderTables,
};

use tp_individual::handler_tables::table::Table;

fn main() {
    // lets read the args
    let args: Vec<String> = std::env::args().collect();

    if let Err(e) = run(args) {
        println!("{}", e);
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

/// Executes the main logical problem of the program
fn run(args: Vec<String>) -> Result<(), Tperrors> {
    if !valid_number_of_args(&args.len()) {
        // we stop the program, invalid number of arguments
        return Err(Tperrors::Generic("Invalid number of arguments".to_string()));
    }

    // Now, we have the arguments...
    let file = &args[1];
    let consult = &args[2].trim();

    let folder_tables = match FolderTables::new(file) {
        Ok(folder_tables) => folder_tables,
        Err(e) => {
            return Err(Tperrors::Table(e.to_string()));
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
                    "Invalid select query (Missing either SELECT , FROM or ;)".to_string(),
                ));
            }

            match resolve_select(&extractor, folder_tables, consult, select) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };
        }
        "INSERT" => {
            let insert = Insert;

            if !insert.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid insert query (Missing either INSERT INTO, VALUES or ;)".to_string(),
                ));
            }

            match resolve_insert(&extractor, folder_tables, consult, insert) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };
        }
        "UPDATE" => {
            let update = Update;

            if !update.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid update query (Missing either UPDATE, SET, WHERE or ;)".to_string(),
                ));
            }

            match resolve_update(&extractor, folder_tables, consult, update) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };
        }
        "DELETE" => {
            let delete = Delete;

            if !delete.is_valid_query(consult) {
                return Err(Tperrors::Syntax(
                    "Invalid delete query (Missing either DELETE, FROM, WHERE or ;)".to_string(),
                ));
            }

            match resolve_delete(&extractor, folder_tables, consult, delete) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            };
        }
        _ => {
            return Err(Tperrors::Syntax("Invalid command".to_string()));
        }
    }
    Ok(())
}

/// Given a consult, command a folder_table instance
///
/// Returns a Table instance to work with
///
/// If the table is not found, returns an error
fn return_proper_table_to_work_with(
    extractor: &Extractor,
    folder_tables: FolderTables,
    consult: &str,
    command: SQLCommand,
) -> Result<Table<File>, Tperrors> {
    let extracted_table_name = match extractor.extract_table(consult, command) {
        Ok(table_name) => table_name,
        Err(e) => {
            return Err(e);
        }
    };

    let table: Table<File> = match folder_tables.get_path(extracted_table_name) {
        Some(table_path) => match Table::<File>::new(table_path) {
            Ok(table) => table,
            Err(e) => {
                return Err(Tperrors::Table(e.to_string()));
            }
        },
        None => {
            return Err(Tperrors::Table("Table not found in the folder".to_string()));
        }
    };

    Ok(table)
}
fn resolve_select(
    extractor: &Extractor,
    folder_tables: FolderTables,
    consult: &str,
    select: Select,
) -> Result<(), Tperrors> {
    let mut table =
        return_proper_table_to_work_with(extractor, folder_tables, consult, SQLCommand::Select)?;

    if !select.is_valid_query(consult) {
        return Err(Tperrors::Syntax(
            "Invalid select query (Missing either SELECT , FROM or ;)".to_string(),
        ));
    }

    // lets get the columns selected from the query
    let columns = match extractor.extract_columns_for_select(consult) {
        Ok(columns) => columns,
        Err(e) => {
            return Err(e);
        }
    };

    // Conditions of the query (if they exists)
    let conditions = extractor.extract_as_str_conditions(consult);

    if let Some(c) = conditions {
        if c.is_empty() {
            return Err(Tperrors::Syntax("incomplete input".to_string()));
        }
    }

    // Sorting method (if existst)
    let sorting_vector = match extractor.extract_orderby_as_str(consult) {
        Some(sorting) => {
            let vec_sort = extractor.parser_orderby_from_str_to_vec(sorting);
            Some(vec_sort)
        }
        None => None,
    };

    // lets execute the query
    select.execute_select(&mut table, columns, conditions, sorting_vector)
}

fn resolve_insert(
    extractor: &Extractor,
    folder_tables: FolderTables,
    consult: &str,
    insert: Insert,
) -> Result<(), Tperrors> {
    let mut table =
        return_proper_table_to_work_with(extractor, folder_tables, consult, SQLCommand::Insert)?;
    if !insert.is_valid_query(consult) {
        return Err(Tperrors::Syntax(
            "Invalid insert query (Missing either INSERT INTO, VALUES or ;)".to_string(),
        ));
    }

    let (columns, values) = match extractor.extract_columns_and_values_for_insert(consult) {
        Ok((columns, values)) => (columns, values),
        Err(e) => {
            return Err(e);
        }
    };
    insert.execute_insert(&mut table, columns, values)
}

fn resolve_update(
    extractor: &Extractor,
    folder_tables: FolderTables,
    consult: &str,
    update: Update,
) -> Result<(), Tperrors> {
    let mut table =
        return_proper_table_to_work_with(extractor, folder_tables, consult, SQLCommand::Update)?;
    if !update.is_valid_query(consult) {
        return Err(Tperrors::Syntax(
            "Invalid update query (Missing either UPDATE, SET, WHERE or ;)".to_string(),
        ));
    }

    let (columns, values) = match extractor.extract_columns_and_values_for_update(consult) {
        Ok((columns, values)) => (columns, values),
        Err(e) => {
            return Err(e);
        }
    };

    let conditions = extractor.extract_as_str_conditions(consult);

    update.execute_update(&mut table, columns, values, conditions)
}

fn resolve_delete(
    extractor: &Extractor,
    folder_tables: FolderTables,
    consult: &str,
    delete: Delete,
) -> Result<(), Tperrors> {
    let mut table =
        return_proper_table_to_work_with(extractor, folder_tables, consult, SQLCommand::Delete)?;

    if !delete.is_valid_query(consult) {
        return Err(Tperrors::Syntax(
            "Invalid delete query (Missing either DELETE, FROM, WHERE or ;)".to_string(),
        ));
    }

    let conditions = extractor.extract_as_str_conditions(consult);

    delete.execute_delete(&mut table, conditions)
}

#[test]
fn run_with_invalid_number_of_args() {
    let args = vec!["".to_string()];
    let result = run(args);
    assert_eq!(result.is_err(), true);
}

#[test]
fn run_invalid_table_throws_error() {
    let args = vec![
        "".to_string(),
        "SELECT * FROM table;".to_string(),
        "table".to_string(),
    ];
    let result = run(args);

    assert_eq!(result.is_err(), true);
}

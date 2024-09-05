pub struct Extractor;

pub struct Condition {
    negation: bool,
    column: String,
    value: String,
}

impl Extractor {
    pub fn new() -> Extractor {
        Extractor
    }

    pub fn extract_columns(&self, query: &String) -> Vec<String> {
        let query = query.trim();

        let where_pos = query.find("FROM");

        let query_whiteplaced = query.split_whitespace().collect::<Vec<&str>>();
        let start = match query_whiteplaced.first() {
            Some(start) => {
                match start {
                    &"SELECT" => "SELECT".len(),
                    &"UPDATE" => "UPDATE".len(),
                    &"DELETE" => "DELETE".len(),
                    &"INSERT" => "INSERT INTO".len(),
                    _ => {
                        // at this point this should never happened because we checked it before.
                        println!("[INVALID_SYNTAX]: Invalid query (Missing SELECT, UPDATE, DELETE or INSERT INTO)");
                        return Vec::new();
                    }
                }
            }
            None => {
                println!("[INVALID_SYNTAX]: Invalid query (Missing SELECT, UPDATE, DELETE or INSERT INTO)");
                return Vec::new();
            }
        };

        match where_pos {
            Some(position_where) => {
                let column_data = &query[start..position_where];
                let column_data = column_data.trim();

                let mut columns: Vec<String> = Vec::new();
                let iterator_columns = column_data.split(",").collect::<Vec<&str>>();

                iterator_columns.into_iter().for_each(|c| {
                    columns.push(c.trim().to_string());
                });

                return columns;
            }
            None => {
                println!("[INVALID_SYNTAX]: Invalid select query (Missing FROM)");
                return Vec::new();
            }
        }
    }

    pub fn extract_table(&self, query: &String) -> String {
        let query = query.trim();

        let from_pos = query.find("FROM");
        let where_or_end_consult_pos = query
            .find("WHERE")
            .or(query.find("ORDER"))
            .or(query.find(";"));
        // we either find WHERE, ORDER and lastly, ;, if not present, query is invalid.

        match (from_pos, where_or_end_consult_pos) {
            (Some(position_from), Some(position_where)) => {
                let table_data = &query[position_from + "FROM".len()..position_where];
                let table_data = table_data.trim();
                println!("Table data: {}", table_data);
                return table_data.to_string();
            }
            _ => {
                if where_or_end_consult_pos.is_none() {
                    println!("[INVALID_SYNTAX]: Invalid select query (Missing WHERE or ;)");
                    return String::new();
                }
                println!("[INVALID_SYNTAX]: Invalid select query (Missing FROM)");
                return String::new();
            }
        }
    }

    pub fn extract_conditions(&self, query: &String) {
        let query = query.trim();

        if let Some(pos) = query.find("WHERE") {
            // we need to concat the vector to that position
            let end = query.find("ORDER").or(query.find(";"));
            let end = match end {
                Some(end) => end,
                None => {
                    println!("[INVALID_SYNTAX]: Invalid select query (Missing ORDER BY or ;)");
                    return;
                }
            };
            let conditions = &query[pos + "WHERE".len()..end];
            let conditions = conditions.trim();
            println!("Conditions: {}", conditions);
        } else {
            // no conditions, but maybe ordered by..

            return;
        };
    }
}

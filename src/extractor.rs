use crate::errors::TPErrors;

pub struct Extractor;

impl Extractor {
    pub fn new() -> Extractor {
        Extractor
    }

    pub fn extract_columns<'a>(&self, query: &'a str) -> Result<Vec<String>, TPErrors<'static>> {
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
                        return Err(TPErrors::InvalidSyntax(
                            "Invalid query (Missing SELECT, UPDATE, DELETE or INSERT INTO)",
                        ));
                    }
                }
            }
            None => {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid query (Missing SELECT, UPDATE, DELETE or INSERT INTO)",
                ));
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

                Ok(columns)
            }
            None => {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid select query (Missing FROM)",
                ));
            }
        }
    }

    pub fn extract_table<'a>(&self, query: &'a str) -> Result<&'a str, TPErrors<'static>> {
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
                Ok(table_data)
            }
            _ => {
                if where_or_end_consult_pos.is_none() {
                    return Err(TPErrors::InvalidSyntax(
                        "Invalid select query (Missing WHERE or ;)",
                    ));
                }
                return Err(TPErrors::InvalidSyntax(
                    "Invalid select query (Missing FROM)",
                ));
            }
        }
    }

    pub fn extract_as_str_conditions<'a>(
        &self,
        query: &'a str,
    ) -> Result<&'a str, TPErrors<'static>> {
        let query = query.trim();

        if let Some(pos) = query.find("WHERE") {
            // we need to concat the vector to that position
            let end = query.find("ORDER").or(query.find(";"));
            let end = match end {
                Some(end) => end,
                None => {
                    return Err(TPErrors::InvalidSyntax(
                        "Invalid select query (Missing ORDER BY or ;)",
                    ));
                }
            };
            let conditions = &query[pos + "WHERE".len()..end];
            let conditions = conditions.trim();
            Ok(conditions)
        } else {
            // no conditions, but maybe ordered by..
            return Err(TPErrors::InvalidSyntax(
                "Invalid select query (Missing WHERE)",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extract_columns() {
        let extractor = Extractor::new();
        let consult: &str = "SELECT name, age FROM table;";
        let columns = extractor.extract_columns(consult).unwrap();

        assert_eq!(columns, vec!["name".to_string(), "age".to_string()]);
    }

    #[test]
    fn extract_table() {
        let extractor = Extractor::new();

        // here i test the table name
        // should return "table" for both cases
        let consults: Vec<&str> = Vec::from([
            "SELECT name, age FROM table;",
            "SELECT name, age FROM table WHERE name = 'John';",
            "SELECT name, are FROM table ORDER BY name;",
        ]);

        for consult in consults {
            let table = extractor.extract_table(consult).unwrap();
            assert_eq!(table, "table");
        }
    }

    #[test]
    fn conditions_query() {
        let extractor = Extractor::new();

        let vec_query: Vec<&str> = vec![
            "SELECT * FROM users WHERE id = 5 AND level = 10;",
            "SELECT * FROM users WHERE id = 5 AND level = 10 ORDER BY id;",
        ];

        for q in vec_query {
            let conditions = extractor.extract_as_str_conditions(q).unwrap();
            assert_eq!(conditions, "id = 5 AND level = 10");
        }
    }
}

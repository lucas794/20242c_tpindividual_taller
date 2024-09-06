use crate::errors::TPErrors;

pub struct Extractor;

impl Extractor {
    pub fn new() -> Extractor {
        Extractor
    }

    /// Given a SQL Consult, we extract the columns as vector of strings.
    /// Example
    /// SELECT name, age FROM table;
    /// Returns ["name", "age"]
    pub fn extract_columns<'a>(&self, query: &'a str) -> Result<Vec<String>, TPErrors<'static>> {
        let query = query.trim();

        let where_pos = query.find("FROM");

        let query_whiteplaced = query.split_whitespace().collect::<Vec<&str>>();
        let start = match query_whiteplaced.first() {
            Some(start) => {
                match *start {
                    "SELECT" => "SELECT".len(),
                    "UPDATE" => "UPDATE".len(),
                    "DELETE" => "DELETE".len(),
                    "INSERT" => "INSERT INTO".len(),
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

    /// Given a SQL Consult, we extract the table name as string.
    /// Example
    /// SELECT * FROM users WHERE id = 3;
    /// Returns "users"
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

    /// Given a SQL Consult, we extract the conditions as string.
    /// Example
    /// SELECT * FROM users WHERE id = 3;
    /// Returns "id = 3"
    pub fn extract_as_str_conditions<'a>(&self, query: &'a str) -> Option<&'a str> {
        let query = query.trim();

        if let Some(pos) = query.find("WHERE") {
            // we need to concat the vector to that position
            let end = query.find("ORDER").or(query.find(";"));
            let end = match end {
                Some(end) => end,
                None => {
                    return None;
                }
            };
            let conditions = &query[pos + "WHERE".len()..end];
            let conditions = conditions.trim();
            Some(conditions)
        } else {
            // no conditions, but maybe ordered by..
            None
        }
    }

    /// Given a query, we extract the ORDER BY columns and if they are ASC or DESC
    pub fn extract_orderby_as_str<'a>(&self, query: &'a str) -> Option<&'a str> {
        let query = query.trim();

        if let Some(pos) = query.find("ORDER BY") {
            let semicolon_end = match query.find(";") {
                Some(end) => end,
                None => {
                    return None;
                }
            };

            let orderby = &query[pos + "ORDER BY".len()..semicolon_end];
            let orderby = orderby.trim();
            Some(orderby)
        } else {
            None
        }
    }

    /// Given a parsed ORDER by clause (previously filtered with extract_orderby_as_str)
    /// Returns a vector of tuples which contains the column to order and how
    /// True means its gonna be ASC, False means its gonna be DESC
    pub fn parser_order_by_str(&self, str_orderby: &str) -> Vec<(String, bool)> {
        str_orderby
            .split(',')
            .map(|part| {
                let parts: Vec<&str> = part.trim().split_whitespace().collect();
                let column = parts[0].to_string();
                // Default to ascending order if no direction is specified
                let asc = if parts.len() == 2 && parts[1].eq("DESC") {
                    false
                } else {
                    true
                };
                (column, asc)
            })
            .collect()
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

    #[test]
    fn orderby_query_without_desc_or_asc() {
        let extractor = Extractor::new();

        let vec_query: Vec<&str> = vec![
            "SELECT * FROM users WHERE id = 5 AND level = 10 ORDER BY id;",
            "SELECT * FROM users ORDER BY id;",
        ];

        for q in vec_query {
            let orderby = extractor.extract_orderby_as_str(q);
            assert_eq!(orderby.is_some(), true);
            assert_eq!(orderby.unwrap(), "id");
        }
    }

    #[test]
    fn orderby_query_with_desc_or_asc() {
        let extractor = Extractor::new();

        let vec_query: Vec<&str> = vec![
            "SELECT * FROM users WHERE id = 5 AND level = 10 ORDER BY id ASC;",
            "SELECT * FROM users ORDER BY id DESC;",
            "SELECT * FROM users ORDER BY id DESC, Nombre ASC;",
        ];

        let expected: Vec<&str> = vec!["id ASC", "id DESC", "id DESC, Nombre ASC"];

        for (i, q) in vec_query.iter().enumerate() {
            let orderby = extractor.extract_orderby_as_str(q).unwrap();
            assert_eq!(orderby, expected[i]);
        }
    }

    #[test]
    fn parser_orderby() {
        let extractor = Extractor::new();

        let vec_query: Vec<&str> = vec![
            "SELECT * FROM users WHERE id = 5 AND level = 10 ORDER BY id ASC;",
            "SELECT * FROM users ORDER BY id DESC;",
            "SELECT * FROM users ORDER BY id DESC, Nombre ASC;",
        ];

        let expected: Vec<Vec<(String, bool)>> = vec![
            vec![("id".to_string(), true)],
            vec![("id".to_string(), false)],
            vec![("id".to_string(), false), ("Nombre".to_string(), true)],
            vec![("id".to_string(), true), ("Nombre".to_string(), false)],
        ];

        for (i, q) in vec_query.iter().enumerate() {
            let orderby = extractor.extract_orderby_as_str(q).unwrap();
            let vec_result = extractor.parser_order_by_str(orderby);
            assert_eq!(vec_result, expected[i]);
        }
    }
}

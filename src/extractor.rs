use crate::errors::TPErrors;

pub struct Extractor;

pub enum SQLCommand {
    SELECT,
    INSERT,
    UPDATE,
    DELETE,
}

impl Extractor {
    pub fn new() -> Extractor {
        Extractor
    }

    /// Given a SQL Consult, we extract the columns as vector of strings.
    /// Example
    /// SELECT name, age FROM table;
    /// Returns ["name", "age"]
    pub fn extract_columns_for_select<'a>(
        &self,
        query: &'a str,
    ) -> Result<Vec<String>, TPErrors<'static>> {
        let query = query.trim();

        let where_pos = query.find("FROM");

        let start = "SELECT".len(); // at this point we know that the first element is SELECT since we validated before.

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

    pub fn extract_columns_and_values_for_insert<'a>(
        &self,
        query: &'a str,
    ) -> Result<(Vec<String>, Vec<String>), TPErrors<'static>> {
        let (start_columns, end_columns) = match (query.find("("), query.find(")")) {
            (Some(start), Some(end)) => (start, end),
            _ => {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid INSERT query (Missing columns)",
                ));
            }
        };

        let (start_values, end_values) = match (query.rfind("("), query.rfind(")")) {
            (Some(start), Some(end)) => (start, end),
            _ => {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid INSERT query (Missing values)",
                ));
            }
        };

        let columns_str = &query[start_columns + 1..end_columns];
        let values_str = &query[start_values + 1..end_values];

        // Parse the columns and values into vectors
        let columns: Vec<String> = columns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let values: Vec<String> = values_str
            .split(',')
            .map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    "".to_string()
                } else {
                    trimmed.trim_matches('\'').to_string()
                }
            })
            .collect();

        // if len doesnt match we return an error
        if columns.len() != values.len() {
            return Err(TPErrors::InvalidSyntax(
                "Invalid INSERT query (Columns and values do not match)",
            ));
        }

        Ok((columns, values))
    }

    /// Given a SQL Consult, we extract the table name as string.
    /// Example
    /// SELECT * FROM users WHERE id = 3;
    /// Returns "users"
    pub fn extract_table<'a>(
        &self,
        query: &'a str,
        consult: SQLCommand,
    ) -> Result<&'a str, TPErrors<'static>> {
        let query = query.trim();

        let (start, offset, end) = self.extract_positions(query, consult);

        match (start, end) {
            (0, 0) => {
                return Err(TPErrors::InvalidSyntax(
                    "Invalid query (Missing any KEY words on your consult)",
                ));
            }
            _ => {
                let table_data = &query[start + offset..end];
                let table_data = table_data.trim();
                Ok(table_data)
            }
        }

        /*let from_pos = query.find("FROM");
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
        }*/
    }

    /// Extracts the position from a QUERY to get the table name
    /// Examples. SELECT * FROM users WHERE id = 3; -> gets FROM as start and WHERE as end, offset will be the length of FROM
    /// INSERT INTO users (name, age) VALUES ('John', 20); -> gets INTO as start and ( as end, offset will be the length of INTO
    /// UPDATE users SET name = 'John' WHERE id = 3; -> gets UPDATE as start and SET as end, offset will be the length of UPDATE
    /// DELETE FROM users WHERE id = 3; -> gets DELETE as start and FROM as end, offset will be the length of DELETE
    fn extract_positions<'a>(&self, query: &'a str, consult: SQLCommand) -> (usize, usize, usize) {
        let query = query.trim();

        let start = match consult {
            SQLCommand::SELECT | SQLCommand::DELETE => match query.find("FROM") {
                Some(pos) => pos,
                None => 0,
            },
            SQLCommand::INSERT => match query.find("INTO") {
                Some(pos) => pos,
                None => 0,
            },
            SQLCommand::UPDATE => match query.find("UPDATE") {
                Some(pos) => pos,
                None => 0,
            },
        };

        let offset = match consult {
            SQLCommand::SELECT => "FROM".len(),
            SQLCommand::INSERT => "INTO".len(),
            SQLCommand::UPDATE => "UPDATE".len(),
            SQLCommand::DELETE => "FROM".len(),
        };

        let end = match consult {
            SQLCommand::SELECT => {
                match query.find("WHERE") {
                    // we need to find WHERE, ORDER or ;
                    Some(pos) => pos,
                    None => {
                        // NO WHERE, so we need to find ORDER or ;
                        match query.find("ORDER") {
                            Some(pos) => pos,
                            None => {
                                // NO ORDER, so we need to find ;
                                match query.find(";") {
                                    Some(pos) => pos,
                                    None => 0, // At this point, this should never happen since we checked before.
                                }
                            }
                        }
                    }
                }
            }
            SQLCommand::INSERT => match query.find("(") {
                Some(pos) => pos,
                None => 0,
            },
            SQLCommand::UPDATE => match query.find("SET") {
                Some(pos) => pos,
                None => 0,
            },
            SQLCommand::DELETE => match query.find("WHERE") {
                Some(pos) => pos,
                None => query.find(";").unwrap(),
            },
        };
        (start, offset, end)
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
    pub fn parser_orderby_from_str_to_vec(&self, str_orderby: &str) -> Vec<(String, bool)> {
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
        let columns = extractor.extract_columns_for_select(consult).unwrap();

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
            let table = extractor
                .extract_table(consult, SQLCommand::SELECT)
                .unwrap();
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
            let vec_result = extractor.parser_orderby_from_str_to_vec(orderby);
            assert_eq!(vec_result, expected[i]);
        }
    }

    #[test]
    fn extract_positions_of_extracting_table_matches() {
        let extractor = Extractor::new();

        let consults: Vec<&str> = Vec::from([
            "SELECT * FROM users WHERE id = 3;",
            "INSERT INTO users (name, age) VALUES ('John', 20);",
            "UPDATE users SET name = 'John' WHERE id = 3;",
            "DELETE FROM users WHERE id = 3;",
        ]);

        // tuples are in (start, offset, end)
        // Example for the first consult
        // FROM is at position 9, so start is 9
        // FROM has a length of 4, so offset is 4
        // WHERE is at position 20, so end is 20
        let expected: Vec<(usize, usize, usize)> =
            vec![(9, 4, 20), (7, 4, 18), (0, 6, 13), (7, 4, 18)];

        let mut start;
        let mut offset;
        let mut end;
        let i = 0;

        (start, offset, end) = extractor.extract_positions(consults[i], SQLCommand::SELECT);
        assert_eq!((start, offset, end), expected[i]);

        (start, offset, end) = extractor.extract_positions(consults[i + 1], SQLCommand::INSERT);
        assert_eq!((start, offset, end), expected[i + 1]);

        (start, offset, end) = extractor.extract_positions(consults[i + 2], SQLCommand::UPDATE);
        assert_eq!((start, offset, end), expected[i + 2]);

        (start, offset, end) = extractor.extract_positions(consults[i + 3], SQLCommand::DELETE);
        assert_eq!((start, offset, end), expected[i + 3]);
    }

    #[test]
    fn extract_table_from_insert_into() {
        let extractor = Extractor::new();

        let consult: &str = "INSERT INTO users (name, age) VALUES ('John', 20);";
        let table = extractor
            .extract_table(consult, SQLCommand::INSERT)
            .unwrap();
        assert_eq!(table, "users");
    }

    #[test]
    fn extract_columns_and_values_from_insert_into() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John', 20);";

        let (columns, values) = extractor
            .extract_columns_and_values_for_insert(consult)
            .unwrap();

        assert_eq!(columns, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(values, vec!["John".to_string(), "20".to_string()]);
    }

    #[test]
    fn extract_columns_and_values_from_insert_into_doesnt_match_fails() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John', 20, 30);";

        let result = extractor.extract_columns_and_values_for_insert(consult);
        assert_eq!(result.is_err(), true);
    }
}

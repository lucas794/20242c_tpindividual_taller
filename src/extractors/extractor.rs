use crate::{errors::tperrors::Tperrors, sorter::sort::SortMethod};

use super::sqlcommand::SQLCommand;

pub struct Extractor;

impl Default for Extractor {
    fn default() -> Self {
        Extractor::new()
    }
}
impl Extractor {
    pub fn new() -> Extractor {
        Extractor
    }

    /// Given a SQL Consult, we extract the columns as vector of strings.
    /// Example
    /// SELECT name, age FROM table;
    /// Returns ["name", "age"]
    pub fn extract_columns_for_select(&self, query: &str) -> Result<Vec<String>, Tperrors> {
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
                    columns.push(c.trim().trim_matches('\'').trim_matches('\"').to_string())
                });
                Ok(columns)
            }
            None => Err(Tperrors::Syntax(
                "Invalid select query (Missing FROM)".to_string(),
            )),
        }
    }

    /// Given a SQL Consult, we extract the columns and values for an INSERT INTO query
    ///
    /// Example
    ///
    /// INSERT INTO users (name, age) VALUES ('John', 20);
    ///
    /// Returns (["name", "age"], ["John", "20"])
    ///
    /// INSERT INTO users (name, age) VALUES ('John', 20), ('Lucas', 'Gabriel');
    ///
    /// Returns (["name", "age"], [["John", "20"], ["Lucas", "Gabriel"]])
    pub fn extract_columns_and_values_for_insert(
        &self,
        query: &str,
    ) -> Result<(Vec<String>, Vec<Vec<String>>), Tperrors> {
        // Step 1: Locate and extract columns
        let (start_columns, end_columns) = match (query.find("("), query.find(")")) {
            (Some(start), Some(end)) if query.contains("VALUES") => (start, end),
            _ => {
                return Err(Tperrors::Syntax(
                    "Invalid INSERT query (Missing columns)".to_string(),
                ));
            }
        };

        let count_single_quotes = query.matches('\'').count();

        if count_single_quotes % 2 != 0 {
            return Err(Tperrors::Syntax(
                "Invalid INSERT query (Unbalanced single quotes)".to_string(),
            ));
        }

        let columns_str = &query[start_columns + 1..end_columns];
        let columns: Vec<String> = columns_str
            .split(',')
            .map(|s| s.trim().trim_matches('\'').trim_matches('\"').to_string())
            .collect();

        // Step 2: Locate the VALUES section and extract multiple value sets
        let values_index = query.find("VALUES").ok_or_else(|| {
            Tperrors::Syntax("Invalid INSERT query (Missing VALUES keyword)".to_string())
        })? + "VALUES".len();

        let values_str = &query[values_index..].trim();

        let mut value_sets = Vec::new();
        let mut remaining_values_str = values_str.to_string(); // Create an owned string to avoid borrowing issues

        while let Some(start) = remaining_values_str.find('(') {
            if let Some(end) = remaining_values_str[start..].find(')') {
                // Extract the value set between parentheses
                let value_set = remaining_values_str[start + 1..start + end]
                    .trim()
                    .to_string();
                value_sets.push(value_set); // Store the value set

                // Update remaining_values_str to start from the character after the closing parenthesis
                remaining_values_str = remaining_values_str[start + end + 1..].trim().to_string();
            } else {
                return Err(Tperrors::Syntax(
                    "Invalid INSERT query (Mismatched parentheses in values)".to_string(),
                ));
            }
        }

        if value_sets.is_empty() {
            return Err(Tperrors::Syntax(
                "Invalid INSERT query (No values found)".to_string(),
            ));
        }

        // Step 4: Parse each value set and ensure it matches the column count
        let mut values: Vec<Vec<String>> = Vec::new();
        for value_set in value_sets {
            let mut current_value = String::new();
            let mut parsed_values = Vec::new();
            let mut in_quotes = false;
            let mut quote_char = ' ';

            for c in value_set.chars() {
                match c {
                    '\'' | '\"' => {
                        if in_quotes {
                            if c == quote_char {
                                in_quotes = false; // Closing the quote
                            } else {
                                current_value.push(c); // Inside a different quote
                            }
                        } else {
                            in_quotes = true; // Starting a quote
                            quote_char = c;
                        }
                    }
                    ',' if !in_quotes => {
                        // Comma separating values outside of quotes
                        parsed_values.push(current_value.trim().to_string());
                        current_value.clear();
                    }
                    _ => {
                        current_value.push(c);
                    }
                }
            }

            // Add the last value
            if !current_value.is_empty() {
                parsed_values.push(current_value.trim().to_string());
            }

            // Ensure the number of values matches the number of columns
            if parsed_values.len() != columns.len() {
                return Err(Tperrors::Syntax(
                    "Invalid INSERT query (Columns and values count mismatch)".to_string(),
                ));
            }

            values.push(parsed_values);
        }

        Ok((columns, values))
    }

    /// This one is tricky.
    ///
    /// Given a SQL Consult, we extract the columns and values for an UPDATE query
    ///
    /// but since update can miss the where condition, we need to handle that case.
    ///
    /// Example: UPDATE users SET name = 'John', age = 20 WHERE id = 3;
    ///
    /// This would return ```(["name", "age"], ["John", "20"])```
    ///
    pub fn extract_columns_and_values_for_update(
        &self,
        query: &str,
    ) -> Result<(Vec<String>, Vec<String>), Tperrors> {
        let (start_columns, end_columns) = match (query.find("SET"), query.find("WHERE")) {
            (Some(start), Some(end)) => (start, end),
            (Some(start), None) => (start, query.len() - 1), // no WHERE, it means ALL tables.., risky..
            _ => {
                return Err(Tperrors::Syntax(
                    "Invalid UPDATE query (Missing SET or WHERE)".to_string(),
                ));
            }
        };

        // lets check unbalanced '
        let count_single_quotes = query.matches('\'').count();

        if count_single_quotes % 2 != 0 {
            return Err(Tperrors::Syntax(
                "Invalid UPDATE query (Unbalanced single quotes)".to_string(),
            ));
        }

        let columns_str = &query[start_columns + "SET".len()..end_columns].trim();

        let tmp_what_to_update: Vec<String> = columns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut columns: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        // we need to split by , and = to get the columns and values
        for what_to_update in tmp_what_to_update {
            let what_to_update = what_to_update.trim();
            let what_to_update = what_to_update.split('=').collect::<Vec<&str>>();

            if what_to_update.len() != 2 {
                return Err(Tperrors::Syntax(
                    "Invalid UPDATE query (Missing =)".to_string(),
                ));
            }

            let column = what_to_update[0]
                .trim()
                .trim_matches('\'')
                .trim_matches('\"')
                .to_string();
            let value = what_to_update[1]
                .trim()
                .trim_matches('\'')
                .trim_matches('\"')
                .to_string();

            columns.push(column);
            values.push(value);
        }
        Ok((columns, values))
    }

    /// Given a SQL Consult, we extract the table name as string.
    ///
    /// Example
    ///
    /// SELECT * FROM users WHERE id = 3;
    ///
    /// Returns "users"
    pub fn extract_table<'a>(
        &self,
        query: &'a str,
        consult: SQLCommand,
    ) -> Result<&'a str, Tperrors> {
        let query = query.trim();

        let (start, offset, end) = self.extract_positions(query, consult);

        match (start, end) {
            (0, 0) => Err(Tperrors::Syntax(
                "Invalid query (Missing any KEY words on your consult)".to_string(),
            )),
            _ => {
                let table_data = &query[start + offset..end];
                let table_data = table_data.trim();
                Ok(table_data)
            }
        }
    }

    /// Extracts the position from a QUERY to get the table name
    ///
    /// Examples. ```SELECT * FROM users WHERE id = 3;``` -> gets FROM as start and WHERE as end, offset will be the length of FROM
    ///
    /// ```INSERT INTO users (name, age) VALUES ('John', 20);``` -> gets INTO as start and ( as end, offset will be the length of INTO
    ///
    /// ```UPDATE users SET name = 'John' WHERE id = 3;``` -> gets UPDATE as start and SET as end, offset will be the length of UPDATE
    ///
    /// ```DELETE FROM users WHERE id = 3;``` -> gets DELETE as start and FROM as end, offset will be the length of DELETE
    ///
    fn extract_positions(&self, query: &str, consult: SQLCommand) -> (usize, usize, usize) {
        let query = query.trim();

        let (start, offset) = match consult {
            SQLCommand::Select | SQLCommand::Delete => {
                let start = query.find("FROM").unwrap_or(0);
                (start, "FROM".len())
            }
            SQLCommand::Insert => {
                let start = query.find("INTO").unwrap_or(0);
                (start, "INTO".len())
            }
            SQLCommand::Update => {
                let start = query.find("UPDATE").unwrap_or(0);
                (start, "UPDATE".len())
            }
        };

        let end = match consult {
            SQLCommand::Select => match query.find("WHERE").or(query.find("ORDER")) {
                Some(pos) => pos,
                None => query.find(";").unwrap_or(0),
            },
            SQLCommand::Insert => {
                //query.find("(").unwrap_or(0)
                let possible_end = query.find("(").unwrap_or(0);
                let values_start = query.find("VALUES").unwrap_or(0);

                if possible_end < values_start {
                    possible_end
                } else {
                    values_start
                }
            }
            SQLCommand::Update => query.find("SET").unwrap_or(0),
            SQLCommand::Delete => match query.find("WHERE") {
                Some(pos) => pos,
                None => query.find(";").unwrap_or(0),
            },
        };
        (start, offset, end)
    }

    /// Given a SQL Consult, we extract the conditions as string.
    ///
    /// Example
    ///
    /// ```SELECT * FROM users WHERE id = 3;```
    /// Returns ```id = 3```
    pub fn extract_as_str_conditions<'a>(&self, query: &'a str) -> Option<&'a str> {
        let query = query.trim();

        if let Some(pos) = query.find("WHERE") {
            // we need to concat the vector to that position
            let end = match query.find("ORDER").or(query.find(";")) {
                Some(end) => end,
                None => {
                    return None;
                }
            };
            let conditions = &query[pos + "WHERE".len()..end].trim();

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

            let orderby = &query[pos + "ORDER BY".len()..semicolon_end].trim();
            Some(orderby)
        } else {
            None
        }
    }

    /// Given a parsed ORDER by clause (previously filtered with extract_orderby_as_str)
    ///
    /// Returns a vector of tuples which contains the column to order and how
    ///
    /// True means its gonna be ASC, False means its gonna be DESC
    ///
    pub fn parser_orderby_from_str_to_vec(&self, str_orderby: &str) -> Vec<SortMethod> {
        str_orderby
            .split(',')
            .map(|part| {
                let parts: Vec<&str> = part.split_whitespace().collect();
                let column = parts[0].to_string();
                // Default to ascending order if no direction is specified
                // clippy witch.
                let asc = !(parts.len() == 2 && parts[1].eq("DESC"));
                SortMethod {
                    by_column: column,
                    ascending: asc,
                }
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
                .extract_table(consult, SQLCommand::Select)
                .unwrap();
            assert_eq!(table, "table");
        }
    }

    #[test]
    fn conditions_multiple_query_with_spaces() {
        let extractor = Extractor::new();

        let vec_query: Vec<&str> = vec![
            "SELECT * FROM users WHERE id = 5 AND level = 10;",
            "SELECT * FROM users WHERE id = 5 AND level = 10 ORDER BY id;",
            "UPDATE users SET name = 'John' WHERE id = 5 AND level = 10;",
        ];

        for q in vec_query {
            let conditions = extractor.extract_as_str_conditions(q).unwrap();
            assert_eq!(conditions, "id = 5 AND level = 10");
        }
    }

    #[test]
    fn conditions_multiple_without_spaces() {
        let extractors = Extractor::new();

        let vec_query: Vec<&str> = vec![
            "SELECT * FROM users WHERE id=5 AND level=10;",
            "SELECT * FROM users WHERE id=5 AND level=10 ORDER BY id;",
            "UPDATE users SET name='John' WHERE id=5 AND level=10;",
        ];

        for q in vec_query {
            let conditions = extractors.extract_as_str_conditions(q).unwrap();
            assert_eq!(conditions, "id=5 AND level=10");
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

        let expected: Vec<Vec<SortMethod>> = vec![
            vec![SortMethod {
                by_column: "id".to_string(),
                ascending: true,
            }],
            vec![SortMethod {
                by_column: "id".to_string(),
                ascending: false,
            }],
            vec![
                SortMethod {
                    by_column: "id".to_string(),
                    ascending: false,
                },
                SortMethod {
                    by_column: "Nombre".to_string(),
                    ascending: true,
                },
            ],
            vec![
                SortMethod {
                    by_column: "id".to_string(),
                    ascending: true,
                },
                SortMethod {
                    by_column: "Nombre".to_string(),
                    ascending: false,
                },
            ],
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

        (start, offset, end) = extractor.extract_positions(consults[i], SQLCommand::Select);
        assert_eq!((start, offset, end), expected[i]);

        (start, offset, end) = extractor.extract_positions(consults[i + 1], SQLCommand::Insert);
        assert_eq!((start, offset, end), expected[i + 1]);

        (start, offset, end) = extractor.extract_positions(consults[i + 2], SQLCommand::Update);
        assert_eq!((start, offset, end), expected[i + 2]);

        (start, offset, end) = extractor.extract_positions(consults[i + 3], SQLCommand::Delete);
        assert_eq!((start, offset, end), expected[i + 3]);
    }

    #[test]
    fn extract_table_from_multiple_consults_matchs() {
        let extractor = Extractor::new();

        let consult_select = "INSERT INTO users (name, age) VALUES ('John', 20);";
        let consult_insert = "SELECT * FROM users WHERE id = 3;";
        let consult_update = "UPDATE users SET name = 'John' WHERE id = 3;";
        let consult_delete = "DELETE FROM users WHERE id = 3;";

        let table_select = extractor
            .extract_table(consult_select, SQLCommand::Insert)
            .unwrap();
        assert_eq!(table_select, "users");

        let table_insert = extractor
            .extract_table(consult_insert, SQLCommand::Select)
            .unwrap();

        assert_eq!(table_insert, "users");

        let table_update = extractor
            .extract_table(consult_update, SQLCommand::Update)
            .unwrap();
        assert_eq!(table_update, "users");

        let table_delete = extractor
            .extract_table(consult_delete, SQLCommand::Delete)
            .unwrap();

        assert_eq!(table_delete, "users");
    }

    #[test]
    fn extract_columns_and_values_from_insert_into() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John', 20);";

        let (columns, values) = extractor
            .extract_columns_and_values_for_insert(consult)
            .unwrap();

        assert_eq!(columns, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(values, [["John".to_string(), "20".to_string()]]);
    }

    #[test]
    fn extract_columns_and_values_from_insert_into_with_multiple_values() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John', 20), ('Lucas', 'Gabriel');";

        let (columns, values) = extractor
            .extract_columns_and_values_for_insert(consult)
            .unwrap();

        assert_eq!(columns, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(
            values,
            [
                ["John".to_string(), "20".to_string()],
                ["Lucas".to_string(), "Gabriel".to_string()]
            ]
        );
    }

    #[test]
    fn extract_columns_and_values_from_insert_into_doesnt_match_fails() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John', 20, 30);";

        let result = extractor.extract_columns_and_values_for_insert(consult);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_columns_with_unbalanced_parentheses_fails() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John', 20;";

        let result = extractor.extract_columns_and_values_for_insert(consult);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_columns_and_values_number_of_columns_and_values_mismatch_fails() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John');";

        let result = extractor.extract_columns_and_values_for_insert(consult);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_columns_and_values_for_update_matches() {
        let extractor = Extractor::new();

        let consult = "UPDATE users SET name = 'John', age = 20 WHERE id = 3;";

        let (columns, values) = extractor
            .extract_columns_and_values_for_update(consult)
            .unwrap();

        assert_eq!(columns, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(values, vec!["John".to_string(), "20".to_string()]);
    }

    #[test]
    fn extract_columns_and_values_for_insert_with_unbalanced_single_quotes_fails() {
        let extractor = Extractor::new();

        let consult = "INSERT INTO users (name, age) VALUES ('John, 20);";

        let result = extractor.extract_columns_and_values_for_insert(consult);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_column_and_values_for_update_with_unbalanced_single_quotes_fails() {
        let extractor = Extractor::new();

        let consult = "UPDATE users SET name = 'John, age = 20 WHERE id = 3;";

        let result = extractor.extract_columns_and_values_for_update(consult);
        assert_eq!(result.is_err(), true);
    }
}

/// Select representation for the SQL query
pub struct Select;
pub struct Insert;
pub struct Update;
pub struct Delete;

/// implementation of the select query
impl Select {
    pub fn new() -> Select {
        Select
    }
    /// A valid select query contains SELECT and FROM
    /// if the query is valid, it will return true
    pub fn is_valid_query<'a>(&self, query: &'a str) -> bool {
        let query = query.trim();

        if query.starts_with("SELECT") && query.contains("FROM") {
            return true;
        }
        false
    }
}

impl Insert {
    pub fn new() -> Insert {
        Insert
    }
}

impl Update {
    pub fn new() -> Update {
        Update
    }
}

impl Delete {
    pub fn new() -> Delete {
        Delete
    }
}

#[cfg(test)]
mod tests {
    use super::Select;

    #[test]
    fn select_invalid_query() {
        let select = Select::new();
        let invalid_consults: Vec<&str> = Vec::from([
            "name, age FROM table",    // missing select
            "SELECT name, age table;", // missing a coma
            "SELECT name, age",        // missing FROM
        ]);
        for invalid_query in invalid_consults {
            assert_eq!(select.is_valid_query(invalid_query), false);
        }
    }
}

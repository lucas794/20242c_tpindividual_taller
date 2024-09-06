/// Select representation for the SQL query
pub struct Select;
pub struct Insert;
pub struct Update;
pub struct Delete;

impl Select {
    pub fn new() -> Select {
        Select
    }

    pub fn is_valid_query(&self, query: &String) -> bool {
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
    fn test_select_invalid_query() {
        let select = Select::new();
        let invalid_consults: Vec<&str> = Vec::from([
            "name, age FROM table",
            "SELECT name, age table;",
            "SELECT name, age",
        ]);
        for invalid_query in invalid_consults {
            assert_eq!(select.is_valid_query(&invalid_query.to_string()), false);
        }
    }
}

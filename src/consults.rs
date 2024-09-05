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

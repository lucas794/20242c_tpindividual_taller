
pub struct Select;

pub struct Insert;
pub struct Update;
pub struct Delete;


trait ExtractorFromConsult {
    fn extract_columns(&self, query: &String) -> Vec<String> {

    }
}


impl Select {
    pub fn new() -> Select {
        Select 
    }

    pub fn is_valid_query(&self, query: &String) -> bool {
        let query = query.to_lowercase();
        let query = query.trim();

        if query.starts_with("select") && query.contains("from") {
            return true;
        }
        false
    }

    pub fn get_query_parts(&self, query: &String) {
        let query = query.to_lowercase();
        let query = query.trim();

        let mut columns: Vec<String> = Vec::new();
        let mut table = String::new();
        let from_pos = query.find("from");
        let where_pos = query.find("where");
        let order_pos = query.find("order by");

        match (from_pos, where_pos, order_pos) {
            (Some(position_from), Some(position_where), Some(order_pos)) => {
                let column_data = &query[String::from("SELECT").len()..position_from];
                let column_data = column_data.trim();
                
                let iterator_columns = column_data.split(",").collect::<Vec<&str>>();
        
                iterator_columns.into_iter().for_each(|c| {
                    columns.push(c.trim().to_string());
                });

                let table_data = &query[position_from + "FROM".len()..position_where];
                let table_data = table_data.trim();


            }
            (Some(position_from), Some(position_where), None) => {
                println!("No order...");
            }
            _ => {
                println!("[INVALID_SYNTAX]: Invalid select query (Missing FROM)");
                return;
            }
        }

        println!("Columns: {:?}", columns);
        println!("Table: ");
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
use std::collections::HashMap;


// tried recursive descent parser but i lost so much time at this point.

/// representation of the type of value can be used for conditions
pub enum Value<'a> {
    /// For now we only support int & string.
    Integer(i64),
    String(&'a str), 
    //String(String), // may be better to use lifetimes i think.
    // pls dont ask for other type.
}

/// representation of the condition that can be used on a query
pub struct Conditions<'a> {
    data: HashMap<String, Value<'a>>,
}

impl<'a> Conditions<'a> {
    pub fn new(data: HashMap<String, Value<'a>>) -> Self {
        Conditions { data }
    }
    pub fn matches_condition(&self, conditions: &str) -> bool {
        let splitted_conditions = conditions.split_whitespace().collect::<Vec<&str>>();

        let mut i = 0;
        let mut result = false;
        let mut negate = false;

        while i < splitted_conditions.len() {
            let token = &splitted_conditions[i];

            match *token {
                "NOT" => {
                    negate = true;
                    i += 1;
                }
                "AND" => {
                    i += 1;
                    result =
                        result && self.evaluate_condition(&splitted_conditions, &mut i, negate);
                }
                "OR" => {
                    i += 1;
                    result =
                        result || self.evaluate_condition(&splitted_conditions, &mut i, negate);
                }
                _ => {
                    result = self.evaluate_condition(&splitted_conditions, &mut i, negate);
                }
            }
            negate = false; // resetting the flag.
        }
        result
    }

    pub fn evaluate_condition(&self, conditions: &Vec<&str>, i: &mut usize, negate: bool) -> bool {
        if *i + 2 >= conditions.len() {
            return false;
        }

        let column = &conditions[*i];
        let operator = &conditions[*i + 1];
        let value = &conditions[*i + 2];
        *i += 3;

        if let Some(actual_value) = self.data.get(*column) {
            let condition_met = match (actual_value, *operator) {
                (Value::String(actual), "=") => actual == &value.trim_matches('\''),
                (Value::String(actual), "!=") => actual != &value.trim_matches('\''),
                (Value::Integer(actual), "=") => {
                    if let Ok(val) = value.parse::<i64>() {
                        actual == &val
                    } else {
                        false
                    }
                }
                (Value::Integer(actual), "!=") => {
                    if let Ok(val) = value.parse::<i64>() {
                        actual != &val
                    } else {
                        false
                    }
                }
                (Value::Integer(actual), ">") => {
                    if let Ok(val) = value.parse::<i64>() {
                        actual > &val
                    } else {
                        false
                    }
                }
                (Value::Integer(actual), "<") => {
                    if let Ok(val) = value.parse::<i64>() {
                        actual < &val
                    } else {
                        false
                    }
                }
                (Value::Integer(actual), ">=") => {
                    if let Ok(val) = value.parse::<i64>() {
                        actual >= &val
                    } else {
                        false
                    }
                }
                (Value::Integer(actual), "<=") => {
                    if let Ok(val) = value.parse::<i64>() {
                        actual <= &val
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if negate {
                !condition_met
            } else {
                condition_met
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_condition_and() {
        let condition_hash: HashMap<String, Value> = HashMap::from([
            ("name".to_string(), Value::String("John")),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec![
            "name = 'John' AND age = 20",
            "age = 20 AND name = 'John'", // backwards should work too..
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }

    #[test]
    fn test_condition_or() {
        let condition_hash: HashMap<String, Value> = HashMap::from([
            ("name".to_string(), Value::String("John")),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec!["name = 'John'", "age = 20 OR name = 'John'"];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }

    #[test]
    fn test_condition_not() {
        let condition_hash: HashMap<String, Value> = HashMap::from([
            ("name".to_string(), Value::String("John")),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec![
            "NOT name = 'John'", // not
            "NOT age = 20",
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }
}

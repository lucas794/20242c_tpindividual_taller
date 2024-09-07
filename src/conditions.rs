// tried recursive descent parser but i lost so much time at this point.
/// representation of the type of value can be used for conditions
pub enum Value {
    /// For now we only support int & string.
    Integer(i64),
    String(String), // tried lifetime, hell no.
                    // pls dont ask for other type.
}

/// representation of the condition that can be used on a query
pub struct Conditions {
    // i got nightmares with &str.. so i will use String
    data: Vec<(String, Value)>,
}

/// implementation of conditions, will be used to check if the conditions are met
impl Conditions {
    pub fn new(data: Vec<(String, Value)>) -> Self {
        Conditions { data }
    }

    /// given a condition as STR it will return if the condition is met
    pub fn matches_condition(&self, conditions: &str) -> bool {
        let splitted_conditions = conditions.split_whitespace().collect::<Vec<&str>>();

        let mut i = 0;
        let mut result = true;
        let mut is_negated = false; // Initialize negation to false

        while i < splitted_conditions.len() {
            let token = &splitted_conditions[i];
            match *token {
                "NOT" => {
                    is_negated = true;
                    i += 1;
                }
                "AND" => {
                    i += 1;
                    result =
                        result && self.evaluate_condition(&splitted_conditions, &mut i, is_negated);
                    is_negated = false; // Reset negation flag after use
                }
                "OR" => {
                    i += 1;
                    result =
                        result || self.evaluate_condition(&splitted_conditions, &mut i, is_negated);
                    is_negated = false; // Reset negation flag after use

                    if result == true {
                        // a single true in OR is enough
                        return true;
                    }
                }
                _ => {
                    result = self.evaluate_condition(&splitted_conditions, &mut i, is_negated);
                    is_negated = false; // Reset negation flag after use
                }
            }
        }
        result
    }

    /// given a condition, it will evaluate if the condition is met
    pub fn evaluate_condition(&self, conditions: &Vec<&str>, i: &mut usize, negate: bool) -> bool {
        if *i + 2 >= conditions.len() {
            return false; // Avoid out-of-bounds access
        }

        let column = &conditions[*i];
        let operator = &conditions[*i + 1];
        let value = &conditions[*i + 2];
        *i += 3; // Advance the index

        // Find the actual value of the column from the record
        let found_column = self.data.iter().find(|(col, _)| col == column);
        if let Some((_, actual_value)) = found_column {
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
            false // Column not found in the record
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn condition_and() {
        let condition_hash: Vec<(String, Value)> = Vec::from([
            ("name".to_string(), Value::String("John".to_string())),
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
    fn condition_or() {
        let condition_hash: Vec<(String, Value)> = Vec::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec!["name = 'John'", "age = 20 OR name = 'John'"];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }

    #[test]
    fn condition_not() {
        let condition_hash: Vec<(String, Value)> =
            Vec::from([("age".to_string(), Value::Integer(20))]);

        let conditions = Conditions::new(condition_hash);

        let str_conditions = vec![
            "NOT age != 20", // not
        ];

        for str_condition in str_conditions {
            assert_ne!(conditions.matches_condition(str_condition), false);
        }
    }

    #[test]
    fn condition_multiple_or_with_same_column() {
        let conditions = Conditions::new(Vec::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]));

        let str_conditions = vec![
            "name = 'John' OR name = 'Marcelo'",
            "age = 20 OR age = 30",
            "name = 'Marcelo' OR age = 20",
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition), true);
        }
    }
}

use std::collections::HashMap;

pub enum Value {
    Integer(i64),
    String(String),
    // pls dont ask for other type.
}

pub struct Conditions {
    data: HashMap<String, Value>,
}

impl Conditions {

    pub fn new(data: HashMap<String, Value>) -> Self {
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

use crate::errors::tperrors::Tperrors;

use super::value::Value;

/// representation of the condition that can be used on a query
pub struct Condition {
    data: Vec<(String, Value)>,
}

/// implementation of conditions, will be used to check if the conditions are met
impl Condition {
    pub fn new(data: Vec<(String, Value)>) -> Self {
        Condition { data }
    }

    /// given a condition as STR it will return if the condition is met
    ///
    /// Assuming conditions holds a condition as example: ```("Name", "John")```
    ///
    /// a query with ```"Name = 'John'"``` will return true
    ///
    pub fn matches_condition(&self, conditions: &str) -> Result<bool, Tperrors> {
        let splitted_conditions = self.preprocess_conditions(conditions);

        if splitted_conditions.len() <= 1 {
            return Err(Tperrors::Syntax(
                "Condition should be separated. Example: Name = 'John'".to_string(),
            ));
        }

        let mut i = 0;
        let mut result_stack: Vec<bool> = vec![];
        let mut operator_stack: Vec<String> = vec![];
        let mut result = true;
        let mut is_negated = false;

        while i < splitted_conditions.len() {
            let token = &splitted_conditions[i];
            match token.as_str() {
                "NOT" => {
                    is_negated = true; // Set negation flag
                    i += 1; // Move to next token, which should be the condition to negate
                }
                "(" => {
                    result_stack.push(result);
                    operator_stack.push("(".to_string());
                    result = true; // Reset result for the expression inside parentheses
                    i += 1;
                }
                ")" => {
                    while let Some(op) = operator_stack.pop() {
                        if op == "(" {
                            break; // Exit when matching parenthesis is found
                        }
                        let previous_result = result_stack
                            .pop()
                            .ok_or(Tperrors::Syntax("Mismatched parentheses".to_string()))?;
                        result = self.combine_with_operator(previous_result, result, &op);
                    }
                    i += 1;
                }
                "AND" | "OR" => {
                    let operator = token.to_string();
                    i += 1; // Move to the next condition after AND/OR
                    let new_result =
                        self.evaluate_condition(&splitted_conditions, &mut i, is_negated)?;

                    // Combine current result with new result
                    result = self.combine_with_operator(result, new_result, &operator);

                    is_negated = false; // Reset negation flag after evaluation
                }
                _ => {
                    // Handle simple condition like "Name = 'John'"
                    result = self.evaluate_condition(&splitted_conditions, &mut i, is_negated)?;
                    is_negated = false; // Reset negation flag after evaluation
                }
            }
        }

        // Handle remaining operators
        while let Some(op) = operator_stack.pop() {
            let previous_result = result_stack.pop().ok_or(Tperrors::Syntax(
                "Mismatched parentheses or missing operators".to_string(),
            ))?;
            result = self.combine_with_operator(previous_result, result, &op);
        }

        Ok(result)
    }

    pub fn evaluate_condition(
        &self,
        conditions: &[String],
        i: &mut usize,
        negate: bool,
    ) -> Result<bool, Tperrors> {
        if *i + 2 >= conditions.len() {
            return Err(Tperrors::Syntax("Condition incomplete".to_string()));
        }

        let column = &conditions[*i].trim_matches('\'').trim_matches('\"');
        let operator = &conditions[*i + 1];
        let value = &conditions[*i + 2].trim_matches('\'').trim_matches('\"');
        *i += 3; // Advance the index properly

        let found_column = self.data.iter().find(|(col, _)| col == column);
        if let Some((_, actual_value)) = found_column {
            let evaluation_check = self.resolve_evaluation(actual_value, operator, value);

            // Apply negation if the negate flag is set
            Ok(if negate {
                !evaluation_check
            } else {
                evaluation_check
            })
        } else {
            if *column == "NOT" {
                *i -= 3;
                return Ok(match negate {
                    true => false,
                    false => true,
                });
            }
            Err(Tperrors::Generic(format!("Error with column {}", column)))
        }
    }

    // Combine results with an operator (AND/OR)
    fn combine_with_operator(&self, lhs: bool, rhs: bool, operator: &str) -> bool {
        match operator {
            "AND" => lhs && rhs,
            "OR" => lhs || rhs,
            _ => rhs, // Default to rhs if no operator is specified
        }
    }

    // Preprocess conditions to properly split parentheses from other tokens
    fn preprocess_conditions(&self, conditions: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut buffer = String::new();

        for ch in conditions.chars() {
            match ch {
                '(' | ')' => {
                    if !buffer.is_empty() {
                        result.push(buffer.clone());
                        buffer.clear();
                    }
                    result.push(ch.to_string());
                }
                ' ' => {
                    if !buffer.is_empty() {
                        result.push(buffer.clone());
                        buffer.clear();
                    }
                }
                _ => buffer.push(ch),
            }
        }

        if !buffer.is_empty() {
            result.push(buffer);
        }

        result
    }

    /*pub fn matches_condition(&self, conditions: &str) -> Result<bool, Tperrors> {
        let splitted_conditions = conditions.split_whitespace().collect::<Vec<&str>>();

        if splitted_conditions.len() <= 1 {
            return Err(Tperrors::Syntax(
                "Condition should be separated. Example: Name = 'John'".to_string(),
            ));
        }

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

                    if !result {
                        // a single false in AND is enough
                        return Ok(false);
                    }
                    is_negated = false; // Reset negation flag after use
                }
                "OR" => {
                    i += 1;
                    result =
                        result || self.evaluate_condition(&splitted_conditions, &mut i, is_negated);

                    is_negated = false; // Reset negation flag after use
                    if result {
                        // a single true in OR is enough
                        return Ok(true);
                    }
                }
                _ => {
                    result = self.evaluate_condition(&splitted_conditions, &mut i, is_negated);
                    is_negated = false; // Reset negation flag after use
                }
            }
        }
        Ok(result)
    }

    /// given a condition, it will evaluate if the condition is met
    pub fn evaluate_condition(&self, conditions: &Vec<String>, i: &mut usize, negate: bool) -> bool {
        if *i + 2 >= conditions.len() {
            return false; // Avoid out-of-bounds access
        }

        let column = &conditions[*i].trim_matches('\'').trim_matches('\"');
        let operator = &conditions[*i + 1];
        let value = &conditions[*i + 2].trim_matches('\'').trim_matches('\"');

        *i += 3; // Advance the index

        // Find the actual value of the column from the record
        let found_column = self.data.iter().find(|(col, _)| col == column);
        if let Some((_, actual_value)) = found_column {
            let evaluation_check = self.resolve_evaluation(actual_value, operator, value);

            if negate {
                !evaluation_check
            } else {
                evaluation_check
            }
        } else {
            false // Column not found in the record
        }
    }*/

    /// Private function that help to check if conditions are met.
    fn resolve_evaluation(&self, actual_value: &Value, operator: &str, value: &str) -> bool {
        match (actual_value, operator) {
            (Value::String(actual), "=") => actual == value.trim_matches('\''),
            (Value::String(actual), "!=") => actual != value.trim_matches('\''),
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

        let conditions = Condition::new(condition_hash);

        let str_conditions = vec![
            "name = 'John' AND age = 20",
            "age = 20 AND name = 'John'", // backwards should work too..
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition).unwrap(), true);
        }
    }

    #[test]
    fn condition_or() {
        let condition_hash: Vec<(String, Value)> = Vec::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]);

        let conditions = Condition::new(condition_hash);

        let str_conditions = vec!["name = 'John'", "age = 20 OR name = 'John'"];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition).unwrap(), true);
        }
    }

    #[test]
    fn condition_not() {
        let condition_hash: Vec<(String, Value)> =
            Vec::from([("age".to_string(), Value::Integer(20))]);

        let conditions = Condition::new(condition_hash);

        let str_conditions = vec![
            "NOT age != 20", // not
        ];

        for str_condition in str_conditions {
            assert_ne!(conditions.matches_condition(str_condition).unwrap(), false);
        }
    }

    #[test]
    fn condition_multiple_or_with_same_column() {
        let conditions = Condition::new(Vec::from([
            ("name".to_string(), Value::String("John".to_string())),
            ("age".to_string(), Value::Integer(20)),
        ]));

        let str_conditions = vec![
            "name = 'John' OR name = 'Marcelo'",
            "age = 20 OR age = 30",
            "name = 'Marcelo' OR age = 20",
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition).unwrap(), true);
        }
    }
}

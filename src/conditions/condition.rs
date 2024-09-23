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
        let tokens = self.preprocess_conditions(conditions);
        let mut i = 0;
        self.evaluate_expression(&tokens, &mut i)
    }

    // Evaluate expressions, handling parentheses and operators recursively
    fn evaluate_expression(&self, tokens: &[String], i: &mut usize) -> Result<bool, Tperrors> {
        let mut operator_stack: Vec<String> = vec![];
        let mut result = true;
        let mut negate_next = false;

        while *i < tokens.len() {
            let token = &tokens[*i];
            match token.as_str() {
                "NOT" => {
                    negate_next = true; // Apply negation
                    *i += 1;
                }
                "(" => {
                    *i += 1; // Skip '('
                    let sub_result = self.evaluate_expression(tokens, i)?;
                    let final_result = if negate_next { !sub_result } else { sub_result };
                    result =
                        self.combine_with_operator(result, final_result, operator_stack.last());
                    negate_next = false;
                }
                ")" => {
                    *i += 1; // Skip ')'
                    return Ok(result);
                }
                "AND" | "OR" => {
                    operator_stack.push(token.to_string());
                    *i += 1;
                }
                _ => {
                    let condition_result = self.evaluate_condition(tokens, i, negate_next)?;
                    result =
                        self.combine_with_operator(result, condition_result, operator_stack.last());
                    negate_next = false;
                }
            }
        }
        Ok(result)
    }

    // Combine two boolean results with the current operator (AND/OR)
    fn combine_with_operator(&self, left: bool, right: bool, operator: Option<&String>) -> bool {
        match operator.map(String::as_str) {
            Some("AND") => left && right,
            Some("OR") => left || right,
            _ => right, // Default to right if no operator is provided (like at start)
        }
    }

    // Evaluate individual condition
    fn evaluate_condition(
        &self,
        tokens: &[String],
        i: &mut usize,
        negate: bool,
    ) -> Result<bool, Tperrors> {
        if *i + 2 >= tokens.len() {
            return Err(Tperrors::Syntax("Condition incomplete".to_string()));
        }

        let column = &tokens[*i].trim_matches('\'').trim_matches('\"');
        let operator = &tokens[*i + 1];
        let value = &tokens[*i + 2].trim_matches('\'').trim_matches('\"');
        *i += 3;

        let found_column = self.data.iter().find(|(col, _)| col == column);
        if let Some((_, actual_value)) = found_column {
            let evaluation_check = self.resolve_evaluation(actual_value, operator, value);
            Ok(if negate {
                !evaluation_check
            } else {
                evaluation_check
            })
        } else {
            Err(Tperrors::Generic(format!(
                "Error with column {}, maybe spaces is required?",
                column
            )))
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

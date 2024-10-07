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
        let open_parenthesis_count = tokens.iter().filter(|x| *x == "(").count();
        let close_parenthesis_count = tokens.iter().filter(|x| *x == ")").count();

        if open_parenthesis_count != close_parenthesis_count {
            return Err(Tperrors::Syntax("near ';'".to_string()));
        }

        let mut i = 0;
        self.evaluate_expression(&tokens, &mut i)
    }

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
            _ => right,
        }
    }

    // Evaluate individual condition
    fn evaluate_condition(
        &self,
        tokens: &[String],
        i: &mut usize,
        negate: bool,
    ) -> Result<bool, Tperrors> {
        // *i + 2 >= tokens.len() || previously.
        if ["AND", "OR", "NOT"].contains(&tokens[tokens.len() - 1].as_str()) {
            return Err(Tperrors::Syntax("Condition incomplete".to_string()));
        }
        // maybe one of the tokens is without whitespaces
        // we need to handle that posibility.
        if tokens.iter().any(|token| {
            (token.contains("=") || token.contains("<") || token.contains(">")) && token.len() > 2
        }) {
            let fixed_tokens = self.split_conditions(tokens)?;
            return self.evaluate_expression(&fixed_tokens, i);
        }
        // lets implement a fix for scaped column or values
        let column = self.return_fixed_column(tokens, i)?;
        *i += 1; // we move the cursor to the operator
        let operator = tokens[*i].to_string().replace('\n', "");
        *i += 1; // we move the cursor to the value
        let value = self.return_fixed_value(tokens, i)?;

        if column.parse::<i64>().is_ok() && value.parse::<i64>().is_ok() {
            // we are evaluating constants here...
            let column = match column.parse::<i64>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(Tperrors::Syntax(
                        "Detected a constant value as column, but error converting it".to_string(),
                    ))
                }
            };

            let value = match value.parse::<i64>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(Tperrors::Syntax(
                        "Detected a constant value as value, but error converting it".to_string(),
                    ))
                }
            };

            let evaluation_check =
                self.resolve_constant_evaluation(column, operator.as_str(), value);

            *i += 1; // we need to go to the next check
            return Ok(if negate {
                !evaluation_check
            } else {
                evaluation_check
            });
        }

        let found_column = self.data.iter().find(|(col, _)| col == column.as_str());
        let found_value_or_column = self.data.iter().find(|(col, _)| col == value.as_str());

        *i += 1;

        if let Some((_, actual_value)) = found_column {
            let evaluation_check = if let Some((_, column_value)) = found_value_or_column {
                self.resolve_column_evaluation(actual_value, operator.as_str(), column_value)
            } else {
                self.resolve_evaluation(actual_value, operator.as_str(), value.as_str())
            };

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
                        result.push(buffer); // Move buffer into result
                        buffer = String::new(); // Reinitialize buffer
                    }
                    result.push(ch.to_string());
                }
                ' ' => {
                    if !buffer.is_empty() {
                        result.push(buffer); // Move buffer into result
                        buffer = String::new(); // Reinitialize buffer
                    }
                }
                _ => buffer.push(ch),
            }
        }

        // After the loop, push any remaining content in the buffer
        if !buffer.is_empty() {
            result.push(buffer);
        }

        result
    }

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

    /// Private function that help to check if conditions are met between columns
    fn resolve_column_evaluation(&self, left: &Value, operator: &str, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(left_val), Value::Integer(right_val)) => match operator {
                "=" => left_val == right_val,
                "!=" => left_val != right_val,
                ">" => left_val > right_val,
                "<" => left_val < right_val,
                ">=" => left_val >= right_val,
                "<=" => left_val <= right_val,
                _ => false,
            },
            (Value::String(left_val), Value::String(right_val)) => match operator {
                "=" | "==" => left_val == right_val,
                "!=" => left_val != right_val,
                _ => false, // String comparisons like ">" are not usually supported
            },
            _ => false,
        }
    }
    // resolves a constant evaluation
    fn resolve_constant_evaluation(&self, left: i64, operator: &str, right: i64) -> bool {
        // we reuse the same function, but we send the values as constants
        self.resolve_column_evaluation(&Value::Integer(left), operator, &Value::Integer(right))
        /*match operator {
            "=" => left == right,
            "!=" => left != right,
            ">" => left > right,
            "<" => left < right,
            ">=" => left >= right,
            "<=" => left <= right,
            _ => false,
        }*/
    }
    fn split_conditions(&self, input: &[String]) -> Result<Vec<String>, Tperrors> {
        let mut result = Vec::new();

        for element in input {
            //for i in 0..input.len() {
            // Check for combined conditions (e.g., "Edad>=45")
            //let element = &input[i];
            // we avoid => or <=
            if element.contains("=>") || element.contains("=<") {
                return Err(Tperrors::Syntax(
                    "Invalid operator, use >= or <= (SQL: Near '<')".to_string(),
                ));
            }

            if element.contains(">=") {
                let parts: Vec<&str> = element.split(">=").collect();
                result.push(parts[0].to_string());
                result.push(">=".to_string());
                result.push(parts[1].to_string());
            } else if element.contains("<=") {
                let parts: Vec<&str> = element.split("<=").collect();
                result.push(parts[0].to_string());
                result.push("<=".to_string());
                result.push(parts[1].to_string());
            } else if element.contains("!=") {
                let parts: Vec<&str> = element.split("!=").collect();
                result.push(parts[0].to_string());
                result.push("!=".to_string());
                result.push(parts[1].to_string());
            } else if element.contains('=') {
                let parts: Vec<&str> = element.split('=').collect();
                if parts.len() == 2 {
                    result.push(parts[0].to_string());
                    result.push("=".to_string());
                    result.push(parts[1].to_string());
                }
            } else if element.contains("<") {
                let parts: Vec<&str> = element.split("<").collect();
                result.push(parts[0].to_string());
                result.push("<".to_string());
                result.push(parts[1].to_string());
            } else if element.contains(">") {
                let parts: Vec<&str> = element.split(">").collect();
                result.push(parts[0].to_string());
                result.push(">".to_string());
                result.push(parts[1].to_string());
            } else {
                // If the element is a logical operator or standalone column
                result.push(element.to_string());
            }
        }

        Ok(result
            .into_iter()
            .filter(|element| !element.is_empty())
            .map(|element| element.trim().to_string())
            .collect::<Vec<String>>())
    }

    /// Given a token and the position of the token, it will return the fixed column
    fn return_fixed_column(&self, tokens: &[String], i: &mut usize) -> Result<String, Tperrors> {
        let result: Result<String, Tperrors> =
            match tokens[*i].contains('\'') || tokens[*i].contains('\"') {
                true => {
                    let find_operator_pos = match tokens.iter().position(|x| {
                        x.contains("=") || x.contains("<") || x.contains(">") || x.contains("!")
                    }) {
                        Some(pos) => pos,
                        None => return Err(Tperrors::Syntax("Comparator error".to_string())),
                    };

                    // the last column scaped SHOULD be scaped, else we throw error syntax
                    match tokens[find_operator_pos - 1].contains("'")
                        || tokens[find_operator_pos - 1].contains("\"")
                    {
                        true => {
                            let fixed_column = tokens[0..find_operator_pos].join(" ");
                            // we make *i jump to the operator
                            *i += find_operator_pos - 1;
                            return Ok(fixed_column
                                .replace("'", "")
                                .replace("\"", "")
                                .replace('\n', "")); //fix for new line
                        }
                        false => return Err(Tperrors::Syntax("Missing scaped column".to_string())),
                    }
                }
                _ => {
                    //*i += 1;
                    Ok(tokens[*i].to_string().replace('\n', ""))
                }
            };
        result
    }

    fn return_fixed_value(&self, tokens: &[String], i: &mut usize) -> Result<String, Tperrors> {
        let result: Result<String, Tperrors> = match tokens[*i].contains('\'')
            || tokens[*i].contains('\"')
        {
            true => {
                let find_pos_operator = tokens[*i..]
                    .iter()
                    .position(|x| x.contains("AND") || x.contains("OR"));

                match find_pos_operator {
                    None | Some(0) => {
                        // this means operator isnt found, so it MUST be the unique and last element of the token
                        let fixed_value = tokens[*i..].join(" ");
                        let distance = (tokens.len() - *i) - 1;
                        let fixed_value = match fixed_value.rfind('\'') {
                            // Idea here is to find the last scape and trim the string to that point.
                            Some(pos) => {
                                if pos == 0 {
                                    // rightest value is at the start of the string?
                                    // this mean isn't scaped.
                                    return Err(Tperrors::Syntax(
                                        "Missing scaped value".to_string(),
                                    ));
                                }
                                fixed_value[1..pos].to_string()
                            }
                            None => fixed_value,
                        };
                        *i += distance; // we need to move the cursor to the distance required
                        Ok(fixed_value
                            .replace("'", "")
                            .replace("\"", "")
                            .replace('\n', ""))
                    }
                    Some(pos) => {
                        // If we found this, it means that there we foun a operator.. we need to check if the previous
                        // value MUST contain a scaped value, if not, we throw an error.
                        if tokens[*i + pos - 1].contains('\'')
                            || tokens[*i + pos - 1].contains('\"')
                        {
                            let fixed_value = tokens[*i..*i + pos].join(" ");

                            *i += pos - 1;
                            Ok(fixed_value
                                .replace("'", "")
                                .replace("\"", "")
                                .replace('\n', ""))
                        } else if tokens[*i + pos - 1] == ")" {
                            // maybe the condition is nested?
                            *i += pos - 1; // we iterate to the next operator
                            Ok(tokens[*i - 1].to_string().replace('\n', "")) //return the last string we found
                        } else {
                            return Err(Tperrors::Syntax(
                                "Missing scaped value, something is wrong".to_string(),
                            ));
                        }
                    }
                }
            }
            false => Ok(tokens[*i].to_string().replace('\n', "")),
        };
        result
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
            "name = 'Marcelo' OR name = 'John'",
            "age = 20 OR age = 30",
            "name = 'Marcelo' OR age = 20",
        ];

        for str_condition in str_conditions {
            assert_eq!(conditions.matches_condition(str_condition).unwrap(), true);
        }
    }
    #[test]
    fn condition_contains_spaces_returns_true() {
        let conditions = Condition::new(Vec::from([(
            "Correo Electronico".to_string(),
            Value::String("test@fi.uba.ar".to_string()),
        )]));

        let condition = "'Correo Electronico'='test@fi.uba.ar'";

        assert_eq!(conditions.matches_condition(condition).unwrap(), true);

        let condition = "'Correo Electronico'=test@fi.uba.ar";

        assert_eq!(conditions.matches_condition(condition).unwrap(), true);
    }
    #[test]
    fn condition_contains_spaces_missing_quote_returns_err() {
        let conditions = Condition::new(Vec::from([(
            "Correo Electronico".to_string(),
            Value::String("test@fi.uba.ar".to_string()),
        )]));

        // we miss in purpose the ' on the column
        let condition = "'Correo Electronico=test@fi.uba.ar";

        assert!(conditions.matches_condition(condition).is_err());

        let conditions = Condition::new(Vec::from([(
            "Correo Electronico".to_string(),
            Value::String("test@fi.uba.ar".to_string()),
        )]));

        // we miss in purpose the ' on the value
        let condition = "'Correo Electronico'='test@fi.uba.ar";

        assert!(conditions.matches_condition(condition).is_err());
    }

    #[test]
    fn conditions_as_constant_resolves_ok() {
        let conditions = Condition::new(Vec::from([("age".to_string(), Value::Integer(20))]));

        let condition = "20 = 20";

        assert_eq!(conditions.matches_condition(condition).unwrap(), true);

        let condition = "20!=20";

        assert_eq!(conditions.matches_condition(condition).unwrap(), false);

        let condition = "20 > 20";

        assert_eq!(conditions.matches_condition(condition).unwrap(), false);

        let condition = "20<20";

        assert_eq!(conditions.matches_condition(condition).unwrap(), false);

        let condition = "20>=20";

        assert_eq!(conditions.matches_condition(condition).unwrap(), true);

        let condition = "20 <= 20";

        assert_eq!(conditions.matches_condition(condition).unwrap(), true);
    }

    #[test]
    fn conditions_advance_consult_with_nested_conditions_contains_unbalanced_scape_throws_error() {
        let conditions = Condition::new(Vec::from([(
            "Correo Electronico".to_string(),
            Value::String("test@fi.uba.ar".to_string()),
        )]));

        let condition = "(Edad>15 AND Nombre='Lucía)";
        // this condition isn't finished, scape missing, so we must throw an error

        assert!(conditions.matches_condition(condition).is_err());
    }

    #[test]
    fn conditions_advance_nested_conditions_contains_scapes_matches() {
        let conditions = Condition::new(Vec::from([
            ("Edad".to_string(), Value::Integer(20)),
            ("Nombre".to_string(), Value::String("Lucía".to_string())),
            ("Nombre".to_string(), Value::String("Paula".to_string())),
        ]));

        let condition = "(Edad > 15 AND Nombre='Lucía') OR Nombre='Paula'";

        assert!(conditions.matches_condition(condition).is_ok());
    }

    #[test]
    fn conditions_without_balanced_parenthesis_fails() {
        let conditions = Condition::new(Vec::from([
            ("Edad".to_string(), Value::Integer(20)),
            ("Nombre".to_string(), Value::String("Lucía".to_string())),
            ("Nombre".to_string(), Value::String("Paula".to_string())),
        ]));

        let condition = "(Edad > 15 AND Nombre='Lucía' OR Nombre='Paula'";

        assert!(conditions.matches_condition(condition).is_err());
    }

    #[test]
    fn conditions_negator_with_scape_returns_matches_ok() {
        let conditions = Condition::new(Vec::from([(
            "Nombre".to_string(),
            Value::String("Lucía".to_string()),
        )]));

        let condition = "NOT Nombre!='Lucía'";

        assert!(conditions.matches_condition(condition).is_ok());
    }
}

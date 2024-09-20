#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        let value = Value::Integer(42);
        assert_eq!(value, Value::Integer(42));

        let value = Value::String("hello".to_string());
        assert_eq!(value, Value::String("hello".to_string()));
    }
}

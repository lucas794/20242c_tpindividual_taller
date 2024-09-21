/// Representation of a value in a condition.
#[derive(Debug)]
pub enum Value {
    Integer(i64),
    String(String),
}

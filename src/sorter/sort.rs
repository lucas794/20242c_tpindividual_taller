use std::fmt::Debug;

/// SortMethod is a struct that holds the information about how to sort the data.
///
/// # Fields
///
/// * `by_column` - The name of the column to sort by.
///
/// * `ascending` - A boolean that determines if the sort should be ascending or descending.
pub struct SortMethod {
    pub by_column: String,
    pub ascending: bool,
}

// lets implement a way to check if its ascending or descending
impl SortMethod {
    /// Returns a boolean that determines if the sort should be ascending or descending.
    ///
    /// # Examples
    ///
    /// ```
    /// use tp_individual::sorter::sort::SortMethod;
    ///
    /// let sort_method = SortMethod {
    ///     by_column: "name".to_string(),
    ///     ascending: true,
    /// };
    ///
    /// assert_eq!(sort_method.is_ascending(), true);
    /// ```
    pub fn is_ascending(&self) -> bool {
        self.ascending
    }

    /// Returns the name of the column to sort by.
    pub fn get_by_column(&self) -> &String {
        &self.by_column
    }
}

impl PartialEq for SortMethod {
    fn eq(&self, other: &Self) -> bool {
        self.by_column == other.by_column && self.ascending == other.ascending
    }
}
impl Debug for SortMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SortMethod {{ by_column: {}, ascending: {} }}",
            self.by_column, self.ascending
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_method() {
        let sort_method = SortMethod {
            by_column: "test_column".to_string(),
            ascending: true,
        };

        assert_eq!(sort_method.get_by_column(), "test_column");
        assert_eq!(sort_method.is_ascending(), true);
    }
}

/// Represents a single text edit operation in the source code.
///
/// An edit specifies a range of bytes to replace and the new content
/// that should be inserted in its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// The byte range to replace (start_byte, end_byte)
    pub range: (usize, usize),
    /// The new content to insert
    pub content: String,
}

/// A target for editing containing a byte range and associated items.
///
/// This structure groups together a range in the source code with
/// items that need to be processed and formatted.
///
/// # Type Parameters
/// * `T` - The type of items being edited (e.g., function arguments, imports)
#[derive(Debug, Clone)]
pub struct EditTarget<T> {
    /// The byte range in the source that this target covers
    pub range: (usize, usize),
    /// The items found within this range that need processing
    pub items: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_creation() {
        let edit = Edit {
            range: (0, 10),
            content: "new content".to_string(),
        };
        assert_eq!(edit.range, (0, 10));
        assert_eq!(edit.content, "new content");
    }

    #[test]
    fn test_edit_equality() {
        let edit1 = Edit {
            range: (0, 5),
            content: "test".to_string(),
        };
        let edit2 = Edit {
            range: (0, 5),
            content: "test".to_string(),
        };
        assert_eq!(edit1, edit2);
    }

    #[test]
    fn test_edit_target_creation() {
        let target: EditTarget<String> = EditTarget {
            range: (0, 20),
            items: vec!["item1".to_string(), "item2".to_string()],
        };
        assert_eq!(target.range, (0, 20));
        assert_eq!(target.items.len(), 2);
    }
}

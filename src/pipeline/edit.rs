/// Represents a text edit operation in the source code.
///
/// An edit specifies a range of bytes to replace. It can either:
/// - Contain final `content` ready to be applied (used in `Pass::run`)
/// - Contain `items` that need to be transformed first (used in `StructuredPass`)
///
/// # Type Parameters
/// * `T` - The type of items being edited (e.g., function arguments, imports).
///   Use `()` for simple edits that only contain content.
///
/// # Examples
/// ```
/// use fmt_runner::Edit;
///
/// // Simple edit with content
/// let edit = Edit::new((0, 10), "new content".to_string());
/// assert_eq!(edit.range, (0, 10));
///
/// // Edit with items to be transformed
/// let edit_target: Edit<String> = Edit::with_items(
///     (0, 20),
///     vec!["item1".to_string(), "item2".to_string()]
/// );
/// assert_eq!(edit_target.items.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct Edit<T = ()> {
    /// The byte range to replace (start_byte, end_byte)
    pub range: (usize, usize),
    /// The new content to insert (used in final edits)
    pub content: Option<String>,
    /// The items found within this range that need processing (used in structured passes)
    pub items: Vec<T>,
}

impl Edit<()> {
    pub fn new(range: (usize, usize), content: String) -> Self {
        Self {
            range,
            content: Some(content),
            items: vec![],
        }
    }
}

impl<T> Edit<T> {
    pub fn with_items(range: (usize, usize), items: Vec<T>) -> Self {
        Self {
            range,
            content: None,
            items,
        }
    }

    pub fn with_content(self, content: String) -> Edit<()> {
        Edit {
            range: self.range,
            content: Some(content),
            items: vec![],
        }
    }
}

impl PartialEq for Edit<()> {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range && self.content == other.content
    }
}

impl Eq for Edit<()> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_creation() {
        let edit = Edit::new((0, 10), "new content".to_string());
        assert_eq!(edit.range, (0, 10));
        assert_eq!(edit.content, Some("new content".to_string()));
    }

    #[test]
    fn test_edit_equality() {
        let edit1 = Edit::new((0, 5), "test".to_string());
        let edit2 = Edit::new((0, 5), "test".to_string());
        assert_eq!(edit1, edit2);
    }

    #[test]
    fn test_edit_target_creation() {
        let target: Edit<String> =
            Edit::with_items((0, 20), vec!["item1".to_string(), "item2".to_string()]);
        assert_eq!(target.range, (0, 20));
        assert_eq!(target.items.len(), 2);
    }
}

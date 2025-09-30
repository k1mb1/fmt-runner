use tree_sitter::Tree;

/// State for parsing, containing source text and optional parse tree.
///
/// This structure maintains the source code and its corresponding parse tree,
/// providing a clean interface for accessing and managing the parsing state.
#[derive(Debug)]
pub struct ParseState {
    pub(crate) source: String,
    pub(crate) tree: Option<Tree>,
    line_offsets: Vec<usize>,
}

impl ParseState {
    /// Create a new parse state with the given source.
    ///
    /// # Arguments
    /// * `source` - The source code to be parsed
    ///
    /// # Examples
    /// ```
    /// use fmt_runner::ParseState;
    ///
    /// let state = ParseState::new("fn main() {}".to_string());
    /// assert_eq!(state.source(), "fn main() {}");
    /// ```
    pub fn new(source: String) -> Self {
        let line_offsets = Self::compute_line_offsets(&source);
        Self {
            source,
            tree: None,
            line_offsets,
        }
    }

    /// Get a reference to the latest parse tree, if any.
    ///
    /// Returns `None` if the source has not been parsed yet.
    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    /// Access the current source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Check if the parse state has a valid tree.
    pub fn has_tree(&self) -> bool {
        self.tree.is_some()
    }

    /// Convert a byte offset to a `tree_sitter::Point`.
    pub(crate) fn byte_to_point(&self, byte: usize) -> tree_sitter::Point {
        let byte = byte.min(self.source.len());

        match self.line_offsets.binary_search(&byte) {
            Ok(line) => tree_sitter::Point {
                row: line,
                column: 0,
            },
            Err(0) => tree_sitter::Point { row: 0, column: byte },
            Err(insert_pos) => {
                let line = insert_pos - 1;
                let line_start = self.line_offsets[line];
                tree_sitter::Point {
                    row: line,
                    column: byte - line_start,
                }
            }
        }
    }

    /// Replace the range in the source, updating the line index.
    pub(crate) fn replace_range(&mut self, range: std::ops::Range<usize>, replacement: &str) {
        self.source.replace_range(range, replacement);
        self.refresh_line_offsets();
    }

    fn refresh_line_offsets(&mut self) {
        self.line_offsets = Self::compute_line_offsets(&self.source);
    }

    fn compute_line_offsets(source: &str) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(source.len() / 16 + 1);
        offsets.push(0);

        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                offsets.push(idx + 1);
            }
        }

        offsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parse_state() {
        let source = "fn main() {}".to_string();
        let state = ParseState::new(source.clone());
        assert_eq!(state.source(), &source);
        assert!(!state.has_tree());
        assert!(state.tree().is_none());
    }

    #[test]
    fn test_has_tree() {
        let state = ParseState::new("test".to_string());
        assert!(!state.has_tree());
    }

    #[test]
    fn test_byte_to_point_single_line() {
        let state = ParseState::new("abcdef".to_string());
        let point = state.byte_to_point(3);
        assert_eq!(point.row, 0);
        assert_eq!(point.column, 3);
    }

    #[test]
    fn test_byte_to_point_multi_line() {
        let state = ParseState::new("ab\ncdef\ng".to_string());
        let point_line1 = state.byte_to_point(3); // newline position -> start of second line
        assert_eq!(point_line1.row, 1);
        assert_eq!(point_line1.column, 0);

    let point_line2 = state.byte_to_point(6);
        assert_eq!(point_line2.row, 1);
    assert_eq!(point_line2.column, 3);

        let point_line3 = state.byte_to_point(8);
        assert_eq!(point_line3.row, 2);
        assert_eq!(point_line3.column, 0);
    }
}

use tree_sitter::Tree;

/// State for parsing, containing source text and optional parse tree.
///
/// This structure maintains the source code and its corresponding parse tree,
/// providing a clean interface for accessing and managing the parsing state.
#[derive(Debug)]
pub struct ParseState {
    pub(crate) source: String,
    pub(crate) tree: Option<Tree>,
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
        Self { source, tree: None }
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
}

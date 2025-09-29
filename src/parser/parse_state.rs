use tree_sitter::Tree;

/// State for parsing, containing source text and optional parse tree.
pub struct ParseState {
    pub(crate) source: String,
    pub(crate) tree: Option<Tree>,
}

impl ParseState {
    /// Create a new parse state with the given source.
    pub fn new(source: String) -> Self {
        Self { source, tree: None }
    }

    /// Get a reference to the latest parse tree, if any.
    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    /// Access the current source text.
    pub fn source(&self) -> &str {
        &self.source
    }
}

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

    /// Get a mutable reference to the source text.
    /// Note: This invalidates the tree, so you'll need to reparse.
    pub fn source_mut(&mut self) -> &mut String {
        self.tree = None; // Invalidate the tree since source will change
        &mut self.source
    }

    /// Replace the whole source.
    pub fn set_source(&mut self, new_source: String) {
        self.source = new_source;
        self.tree = None;
    }

    /// Get the length of the source text.
    pub fn len(&self) -> usize {
        self.source.len()
    }

    /// Check if the source text is empty.
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
}

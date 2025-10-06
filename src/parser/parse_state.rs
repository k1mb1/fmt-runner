use tree_sitter::Tree;

#[derive(Debug)]
pub struct ParseState {
    pub(crate) source: String,
    pub(crate) tree: Option<Tree>,
}

impl ParseState {
    pub fn new(source: String) -> Self {
        Self { source, tree: None }
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

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

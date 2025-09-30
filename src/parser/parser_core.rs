use crate::parser::language_provider::LanguageProvider;
use crate::parser::parse_state::ParseState;
use tree_sitter::{InputEdit, Parser as TsParser};

/// Generic parser that owns a tree-sitter parser.
/// The source and tree are managed separately in ParseState.
pub struct Parser<Language: LanguageProvider> {
    ts_parser: TsParser,
    _marker: std::marker::PhantomData<Language>,
}

impl<Language: LanguageProvider> Parser<Language> {
    /// Create a new parser for the language.
    pub fn new() -> Self {
        let mut ts_parser = TsParser::new();
        ts_parser
            .set_language(&Language::language())
            .expect("Error loading grammar");

        Self {
            ts_parser,
            _marker: std::marker::PhantomData,
        }
    }

    /// Parse the source in the state from scratch.
    pub fn parse(&mut self, state: &mut ParseState) {
        state.tree = self.ts_parser.parse(&state.source, None);
    }

    /// Incrementally reparse using the existing tree (if any).
    pub fn reparse(&mut self, state: &mut ParseState) {
        state.tree = self.ts_parser.parse(&state.source, state.tree.as_ref());
    }

    /// Apply an edit to the source in the state and update tree-sitter's tree edit before reparsing.
    ///
    /// `start_byte..old_end_byte` will be replaced with `new_text`.
    pub fn apply_edit(
        &mut self,
        state: &mut ParseState,
        start_byte: usize,
        old_end_byte: usize,
        new_text: &str,
    ) {
        let start_point = state.byte_to_point(start_byte);
        let old_end_point = state.byte_to_point(old_end_byte);

        let new_end_byte = start_byte + new_text.len();

        state.replace_range(start_byte..old_end_byte, new_text);

        let new_end_point = state.byte_to_point(new_end_byte);

        if let Some(tree) = &mut state.tree {
            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: new_end_point,
            };
            tree.edit(&edit);
        }
    }
}

impl<Language: LanguageProvider> Default for Parser<Language> {
    fn default() -> Self {
        Self::new()
    }
}

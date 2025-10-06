use crate::parser::language_provider::LanguageProvider;
use crate::parser::parse_state::ParseState;
use tree_sitter::{InputEdit, Parser as TsParser};

pub struct Parser<Language: LanguageProvider> {
    ts_parser: TsParser,
    _marker: std::marker::PhantomData<Language>,
}

impl<Language: LanguageProvider> Parser<Language> {
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

    pub fn parse(&mut self, state: &mut ParseState) {
        state.tree = self.ts_parser.parse(&state.source, None);
    }

    pub fn reparse(&mut self, state: &mut ParseState) {
        state.tree = self.ts_parser.parse(&state.source, state.tree.as_ref());
    }

    pub fn apply_edit(
        &mut self,
        state: &mut ParseState,
        start_byte: usize,
        old_end_byte: usize,
        new_text: &str,
    ) {
        state
            .source
            .replace_range(start_byte..old_end_byte, new_text);
        if let Some(tree) = &mut state.tree {
            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte: start_byte + new_text.len(),
                start_position: tree_sitter::Point {
                    row: 0,
                    column: start_byte,
                },
                old_end_position: tree_sitter::Point {
                    row: 0,
                    column: old_end_byte,
                },
                new_end_position: tree_sitter::Point {
                    row: 0,
                    column: start_byte + new_text.len(),
                },
            };
            tree.edit(&edit);
        }
        self.reparse(state);
    }
}

impl<Language: LanguageProvider> Default for Parser<Language> {
    fn default() -> Self {
        Self::new()
    }
}

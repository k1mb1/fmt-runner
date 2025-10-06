use crate::parser::{LanguageProvider, ParseState, Parser};
use crate::pipeline::Pipeline;
use log::debug;
use std::marker::PhantomData;
use std::path::PathBuf;

// ...existing code...
pub struct Engine<Language: LanguageProvider, Config> {
    pipeline: Pipeline<Config>,
    parser: Parser<Language>,
    _marker: PhantomData<(Language, Config)>,
}

impl<Language: LanguageProvider, C> Engine<Language, C> {
    pub fn new(pipeline: Pipeline<C>) -> Self {
        Self {
            pipeline,
            parser: Parser::new(),
            _marker: PhantomData,
        }
    }

    fn run(&mut self, config: &C, state: &mut ParseState) {
        // Ensure we have a parsed tree
        if !state.has_tree() {
            self.parser.parse(state);
        }

        // Apply each pass in the pipeline
        for pass in self.pipeline.passes() {
            let root = state
                .tree()
                .expect("Tree should exist after parsing")
                .root_node();
            let source = state.source();

            let mut edits = pass.run(config, &root, source);
            debug!("Pass generated {} edit(s)", edits.len());

            // Sort edits in reverse order to maintain byte offsets
            edits.sort_by(|a, b| b.range.0.cmp(&a.range.0));

            // Apply each edit
            for edit in edits {
                debug!("Applying edit at range {:?}", edit.range);
                let content = edit
                    .content
                    .expect("Edit should have content after pass.run()");
                self.parser
                    .apply_edit(state, edit.range.0, edit.range.1, &content);
            }
        }
    }

    pub fn check(&mut self, config: &C, codes: &[String], files: &[PathBuf]) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();

        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.clone());
            self.run(config, &mut state);

            if state.source() != code && i < files.len() {
                changed_files.push(files[i].clone());
            }
        }

        changed_files
    }

    pub fn format_and_write(
        &mut self,
        config: &C,
        codes: &[String],
        files: &[PathBuf],
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut changed_files = Vec::new();

        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.clone());
            self.run(config, &mut state);

            let formatted_code = state.source();
            if formatted_code != code && i < files.len() {
                let file_path = &files[i];
                std::fs::write(file_path, formatted_code)?;
                changed_files.push(file_path.clone());
            }
        }

        Ok(changed_files)
    }
}

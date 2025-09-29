use crate::parser::{LanguageProvider, ParseState, Parser};
use crate::pipeline::Pipeline;
use std::marker::PhantomData;
use std::path::PathBuf;

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

    /// Get a reference to the pipeline
    pub fn pipeline(&self) -> &Pipeline<C> {
        &self.pipeline
    }

    /// Get a mutable reference to the pipeline
    pub fn pipeline_mut(&mut self) -> &mut Pipeline<C> {
        &mut self.pipeline
    }

    /// Get a reference to the parser
    pub fn parser(&self) -> &Parser<Language> {
        &self.parser
    }

    /// Get a mutable reference to the parser
    pub fn parser_mut(&mut self) -> &mut Parser<Language> {
        &mut self.parser
    }

    fn run(&mut self, config: &C, state: &mut ParseState) {
        if state.tree().is_none() {
            self.parser.parse(state);
        }

        for pass in self.pipeline.passes() {
            let root = state.tree().unwrap().root_node();
            let source = state.source();

            let mut edits = pass.run(config, &root, source);
            println!("Edits for pass: {:?}", edits);

            edits.sort_by(|a, b| b.range.0.cmp(&a.range.0));

            for edit in edits {
                self.parser
                    .apply_edit(state, edit.range.0, edit.range.1, &edit.content);
            }
        }
    }

    pub fn start(&mut self, config: &C, codes: &[String]) {
        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.to_string());
            self.run(config, &mut state);

            println!("Final source for code {}:", i + 1);
            println!("{}", state.source());
            println!("---");
        }
    }

    /// Check if files need formatting (returns list of files that would be changed)
    pub fn check(&mut self, config: &C, codes: &[String], files: &[PathBuf]) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();

        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.to_string());
            self.run(config, &mut state);

            if state.source() != code {
                if i < files.len() {
                    changed_files.push(files[i].clone());
                }
            }
        }

        changed_files
    }

    /// Format files and write changes (returns list of files that were changed)
    pub fn format_and_write(
        &mut self,
        config: &C,
        codes: &[String],
        files: &[PathBuf],
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut changed_files = Vec::new();

        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.to_string());
            self.run(config, &mut state);

            let formatted_code = state.source();
            if formatted_code != code {
                if i < files.len() {
                    let file_path = &files[i];
                    std::fs::write(file_path, formatted_code)?;
                    changed_files.push(file_path.clone());
                }
            }
        }

        Ok(changed_files)
    }
}

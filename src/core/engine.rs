use crate::parser::{LanguageProvider, ParseState, Parser};
use crate::pipeline::Pipeline;
use log::debug;
use std::marker::PhantomData;
use std::path::PathBuf;

/// The main formatting engine that coordinates parsing and pipeline execution.
///
/// The engine manages a parser and a pipeline of formatting passes, applying
/// them to source code to produce formatted output.
///
/// # Type Parameters
/// * `Language` - A type implementing `LanguageProvider` for language-specific parsing
/// * `Config` - Configuration type passed to formatting passes
///
/// # Examples
/// ```ignore
/// let pipeline = Pipeline::new();
/// let mut engine = Engine::<MyLanguage, MyConfig>::new(pipeline);
/// ```
pub struct Engine<Language: LanguageProvider, Config> {
    pipeline: Pipeline<Config>,
    parser: Parser<Language>,
    _marker: PhantomData<(Language, Config)>,
}

impl<Language: LanguageProvider, C> Engine<Language, C> {
    /// Create a new engine with the given pipeline.
    ///
    /// # Arguments
    /// * `pipeline` - The formatting pipeline to use
    pub fn new(pipeline: Pipeline<C>) -> Self {
        Self {
            pipeline,
            parser: Parser::new(),
            _marker: PhantomData,
        }
    }

    /// Run the pipeline on the given parse state.
    ///
    /// This method applies all passes in the pipeline sequentially,
    /// collecting edits and applying them in reverse order to maintain
    /// correct byte offsets.
    ///
    /// # Arguments
    /// * `config` - Configuration to pass to each pass
    /// * `state` - The parse state containing source and tree
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

    /// Check if files need formatting (returns list of files that would be changed).
    ///
    /// This method runs the pipeline on each file and compares the result
    /// with the original content without writing changes to disk.
    ///
    /// # Arguments
    /// * `config` - Configuration to pass to formatting passes
    /// * `codes` - Source code contents of the files
    /// * `files` - File paths corresponding to the source codes
    ///
    /// # Returns
    /// A vector of file paths that would be changed by formatting
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

    /// Format files and write changes (returns list of files that were changed).
    ///
    /// This method runs the pipeline on each file, writes the formatted
    /// content to disk if it differs from the original, and returns the
    /// list of modified files.
    ///
    /// # Arguments
    /// * `config` - Configuration to pass to formatting passes
    /// * `codes` - Source code contents of the files
    /// * `files` - File paths corresponding to the source codes
    ///
    /// # Returns
    /// A `Result` containing a vector of changed file paths, or an IO error
    ///
    /// # Errors
    /// Returns an error if writing to any file fails
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

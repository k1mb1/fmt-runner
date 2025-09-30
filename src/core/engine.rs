use super::{Diagnostic, DiagnosticSeverity, FileFormatOutcome};
use crate::parser::{LanguageProvider, ParseState, Parser};
use crate::pipeline::{Edit, FormatterContext, Pipeline};
use similar::TextDiff;
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
    fn run(&mut self, config: &C, state: &mut ParseState) -> RunReport {
        if !state.has_tree() {
            self.parser.parse(state);
        }

    let original_source = state.source().to_string();
        let mut diagnostics = Vec::new();

        for (index, pass) in self.pipeline.passes().iter().enumerate() {
            let root = state
                .tree()
                .expect("Tree should exist after parsing")
                .root_node();
            let source = state.source();

            let mut context = FormatterContext::new(config, root, source)
                .with_default_source(format!("pass#{index}"));

            let mut edits = pass.run(&mut context);
            let mut pass_diags = context.into_diagnostics();
            debug!("Pass generated {} edit(s)", edits.len());

            if edits.is_empty() {
                diagnostics.append(&mut pass_diags);
                continue;
            }

            if let Err(conflict) = normalize_edits(&mut edits) {
                let message = format!(
                    "Conflicting edits detected between ranges {:?} and {:?}",
                    conflict.first, conflict.second
                );
                let conflict_diag = Diagnostic {
                    range: Some(conflict.first),
                    message,
                    severity: DiagnosticSeverity::Error,
                    source: Some(format!("pass#{index}")),
                };
                diagnostics.push(conflict_diag);
                diagnostics.append(&mut pass_diags);

                log::error!(
                    "Conflicting edits detected in pass #{index}: {:?} vs {:?}",
                    conflict.first,
                    conflict.second
                );
                continue;
            }

            let mut applied_any = false;

            for edit in edits.into_iter().rev() {
                debug!("Applying edit at range {:?}", edit.range);
                self.parser
                    .apply_edit(state, edit.range.0, edit.range.1, &edit.content);
                applied_any = true;
            }

            if applied_any {
                self.parser.reparse(state);
            }
            diagnostics.append(&mut pass_diags);
        }

        let final_source = state.source().to_string();
        let changed = final_source != original_source;

        RunReport {
            changed,
            diagnostics,
            final_source,
        }
    }

    /// Check if files need formatting and collect diagnostics.
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
    /// A vector of outcomes describing the effect of formatting on each file
    pub fn check(
        &mut self,
        config: &C,
        codes: &[String],
        files: &[PathBuf],
    ) -> Vec<FileFormatOutcome> {
        let mut outcomes = Vec::with_capacity(codes.len());

        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.clone());
            let report = self.run(config, &mut state);
            let RunReport {
                changed,
                diagnostics,
                final_source,
            } = report;
            let path = files.get(i).cloned();
            let mut outcome = FileFormatOutcome::new(path);
            outcome.changed = changed;
            outcome.diagnostics = diagnostics;
            outcome.diff = compute_diff(code, &final_source);

            outcomes.push(outcome);
        }

        outcomes
    }

    /// Format files, write changes, and collect diagnostics.
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
    /// A `Result` containing a vector of outcomes for each file, or an IO error
    ///
    /// # Errors
    /// Returns an error if writing to any file fails
    pub fn format_and_write(
        &mut self,
        config: &C,
        codes: &[String],
        files: &[PathBuf],
    ) -> Result<Vec<FileFormatOutcome>, std::io::Error> {
        let mut outcomes = Vec::with_capacity(codes.len());

        for (i, code) in codes.iter().enumerate() {
            let mut state = ParseState::new(code.clone());
            let report = self.run(config, &mut state);
            let RunReport {
                changed,
                diagnostics,
                final_source,
            } = report;
            let path = files.get(i).cloned();
            let mut outcome = FileFormatOutcome::new(path.clone());
            outcome.changed = changed;
            outcome.diagnostics = diagnostics;
            outcome.diff = compute_diff(code, &final_source);

            if outcome.changed {
                if let Some(file_path) = path {
                    std::fs::write(&file_path, final_source.as_bytes())?;
                }
            }

            outcomes.push(outcome);
        }

        Ok(outcomes)
    }
}

struct RunReport {
    changed: bool,
    diagnostics: Vec<Diagnostic>,
    final_source: String,
}

#[derive(Debug, Clone, Copy)]
struct EditConflict {
    first: (usize, usize),
    second: (usize, usize),
}

fn normalize_edits(edits: &mut Vec<Edit>) -> Result<(), EditConflict> {
    edits.sort_by(|a, b| a.range.0.cmp(&b.range.0));

    for window in edits.windows(2) {
        let first = &window[0];
        let second = &window[1];

        if first.range.1 > second.range.0 {
            return Err(EditConflict {
                first: first.range,
                second: second.range,
            });
        }
    }

    Ok(())
}

fn compute_diff(original: &str, formatted: &str) -> Option<String> {
    if original == formatted {
        return None;
    }

    let diff = TextDiff::from_lines(original, formatted);
    let unified = diff
        .unified_diff()
        .context_radius(3)
        .header("original", "formatted")
        .to_string();

    Some(unified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_edits_accepts_non_overlapping() {
        let mut edits = vec![
            Edit {
                range: (0, 5),
                content: "foo".to_string(),
            },
            Edit {
                range: (10, 12),
                content: "bar".to_string(),
            },
        ];

        assert!(normalize_edits(&mut edits).is_ok());
        assert_eq!(edits[0].range, (0, 5));
        assert_eq!(edits[1].range, (10, 12));
    }

    #[test]
    fn normalize_edits_detects_overlap() {
        let mut edits = vec![
            Edit {
                range: (0, 5),
                content: "foo".to_string(),
            },
            Edit {
                range: (4, 6),
                content: "bar".to_string(),
            },
        ];

        let conflict = normalize_edits(&mut edits).expect_err("expected conflict");
        assert_eq!(conflict.first, (0, 5));
        assert_eq!(conflict.second, (4, 6));
    }

    #[test]
    fn compute_diff_returns_none_for_equal_strings() {
        assert!(compute_diff("abc", "abc").is_none());
    }

    #[test]
    fn compute_diff_returns_unified_diff_for_changes() {
        let diff = compute_diff("a\n", "b\n").expect("diff expected");
        assert!(diff.contains("-a"));
        assert!(diff.contains("+b"));
    }
}

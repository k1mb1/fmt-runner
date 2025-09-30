use crate::core::{Diagnostic, DiagnosticSeverity};
use tree_sitter::Node;

/// Shared context provided to passes during execution.
pub struct FormatterContext<'config, 'tree, Config> {
    config: &'config Config,
    root: Node<'tree>,
    source: &'config str,
    diagnostics: Vec<Diagnostic>,
    default_source: Option<String>,
}

impl<'config, 'tree, Config> FormatterContext<'config, 'tree, Config> {
    pub(crate) fn new(
        config: &'config Config,
        root: Node<'tree>,
        source: &'config str,
    ) -> Self {
        Self {
            config,
            root,
            source,
            diagnostics: Vec::new(),
            default_source: None,
        }
    }

    pub(crate) fn with_default_source(mut self, source: String) -> Self {
        self.default_source = Some(source);
        self
    }

    /// Access the configuration shared across passes.
    pub fn config(&self) -> &'config Config {
        self.config
    }

    /// Access the AST root node for the current file.
    pub fn root(&self) -> Node<'tree> {
        self.root
    }

    /// Access the current source text.
    pub fn source(&self) -> &'config str {
        self.source
    }

    /// Register an arbitrary diagnostic.
    pub fn push_diagnostic(&mut self, mut diagnostic: Diagnostic) {
        if diagnostic.source.is_none() {
            diagnostic.source = self.default_source.clone();
        }
        self.diagnostics.push(diagnostic);
    }

    /// Convenience helper for emitting an info diagnostic.
    pub fn info(&mut self, message: impl Into<String>, range: Option<(usize, usize)>) {
        self.push_diagnostic(Diagnostic {
            range,
            message: message.into(),
            severity: DiagnosticSeverity::Info,
            source: None,
        });
    }

    /// Convenience helper for emitting a warning diagnostic.
    pub fn warning(&mut self, message: impl Into<String>, range: Option<(usize, usize)>) {
        self.push_diagnostic(Diagnostic {
            range,
            message: message.into(),
            severity: DiagnosticSeverity::Warning,
            source: None,
        });
    }

    /// Convenience helper for emitting an error diagnostic.
    pub fn error(&mut self, message: impl Into<String>, range: Option<(usize, usize)>) {
        self.push_diagnostic(Diagnostic {
            range,
            message: message.into(),
            severity: DiagnosticSeverity::Error,
            source: None,
        });
    }

    pub(crate) fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

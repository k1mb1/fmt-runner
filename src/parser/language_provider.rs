use crate::supported_extension::SupportedExtension;
use tree_sitter::Language;

/// A trait for providing language-specific information for parsing.
///
/// Implement this trait to define a new language that can be parsed
/// and formatted by the engine. The trait is typically implemented
/// on zero-sized types (unit structs).
///
/// # Examples
/// ```ignore
/// use tree_sitter::Language;
/// use fmt_runner::{LanguageProvider, SupportedExtension};
///
/// struct RustLanguage;
///
/// impl LanguageProvider for RustLanguage {
///     fn language() -> Language {
///         tree_sitter_rust::language()
///     }
///
///     fn supported_extension() -> &'static SupportedExtension {
///         static RUST_EXTENSIONS: SupportedExtension = SupportedExtension::new(&["rs"]);
///         &RUST_EXTENSIONS
///     }
/// }
/// ```
pub trait LanguageProvider {
    /// Get the tree-sitter Language for this language.
    ///
    /// This method returns the tree-sitter grammar definition that will
    /// be used to parse source code.
    fn language() -> Language;

    /// Get the supported file extensions for this language.
    ///
    /// Returns a reference to a static `SupportedExtension` that defines
    /// which file extensions should be processed by this language's formatter.
    fn supported_extension() -> &'static SupportedExtension;
}

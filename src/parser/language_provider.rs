use crate::supported_extension::SupportedExtension;
use tree_sitter::Language;

/// A trait implemented by a zero-sized type that provides a tree-sitter `Language`.
pub trait LanguageProvider {
    fn language() -> Language;
    fn supported_extension() -> &'static SupportedExtension;
}

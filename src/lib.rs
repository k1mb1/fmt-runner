pub mod cli;
pub mod core;
pub mod parser;
pub mod pipeline;
pub mod supported_extension;

// Re-export commonly used items for convenience
pub use cli::{cli_builder, AdvancedCliBuilder, CliBuilder};
pub use parser::LanguageProvider;
pub use pipeline::{Edit, Pass, Pipeline, StructuredPass};
pub use supported_extension::SupportedExtension;

use fmt_runner::{cli_builder, Edit, LanguageProvider, Pass, Pipeline, SupportedExtension};
use log::info;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

/// Example configuration for the formatter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MyConfig {
    pub indent_size: usize,
    pub max_line_length: usize,
}

/// Example pass for indentation
struct IndentationPass;

impl Pass for IndentationPass {
    type Config = MyConfig;

    fn run(&self, config: &Self::Config, _root: &Node, _source: &str) -> Vec<Edit> {
        // Example implementation - in real code you'd analyze the AST
        info!(
            "Running indentation pass with indent_size: {}",
            config.indent_size
        );
        Vec::new()
    }
}

/// Example pass for line length
struct LineLengthPass;

impl Pass for LineLengthPass {
    type Config = MyConfig;

    fn run(&self, config: &Self::Config, _root: &Node, _source: &str) -> Vec<Edit> {
        // Example implementation - in real code you'd analyze the AST
        info!(
            "Running line length pass with max_line_length: {}",
            config.max_line_length
        );
        Vec::new()
    }
}

/// Example language provider
struct MyLanguage;

static MY_EXTENSIONS: SupportedExtension = SupportedExtension::new(&["my", "mylang"]);

impl LanguageProvider for MyLanguage {
    fn language() -> tree_sitter::Language {
        // This would be your actual tree-sitter language
        // For example purposes, we'll use a placeholder
        // You would need to add the appropriate tree-sitter language dependency
        unimplemented!("Add your tree-sitter language here")
    }

    fn supported_extension() -> &'static SupportedExtension {
        &MY_EXTENSIONS
    }
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Example 1: Using add_pass method chaining
    cli_builder::<MyLanguage, MyConfig>()
        .add_pass(IndentationPass)
        .add_pass(LineLengthPass)
        .run();

    // Example 2: Using with_pipeline
    let mut pipeline = Pipeline::<MyConfig>::new();
    pipeline.add_pass(IndentationPass);
    pipeline.add_pass(LineLengthPass);

    cli_builder::<MyLanguage, MyConfig>()
        .with_pipeline(pipeline)
        .run();

    // Example 3: Creating a reusable builder factory
    let create_cli = || {
        cli_builder::<MyLanguage, MyConfig>()
            .add_pass(IndentationPass)
            .add_pass(LineLengthPass)
    };

    create_cli().run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmt_runner::CliBuilder;

    #[test]
    fn test_builder_creation() {
        let builder = CliBuilder::<MyLanguage, MyConfig>::new();
        // Builder should be created successfully
        assert!(std::ptr::eq(&builder as *const _, &builder as *const _));
    }

    #[test]
    fn test_builder_with_passes() {
        let builder = CliBuilder::<MyLanguage, MyConfig>::new()
            .add_pass(IndentationPass)
            .add_pass(LineLengthPass);

        // Builder should accept passes
        assert!(std::ptr::eq(&builder as *const _, &builder as *const _));
    }

    #[test]
    fn test_builder_with_pipeline() {
        let pipeline = Pipeline::<MyConfig>::new();
        let builder = CliBuilder::<MyLanguage, MyConfig>::new().with_pipeline(pipeline);

        // Builder should accept pipeline
        assert!(std::ptr::eq(&builder as *const _, &builder as *const _));
    }

    #[test]
    fn test_convenience_function() {
        let builder = cli_builder::<MyLanguage, MyConfig>().add_pass(IndentationPass);

        // Convenience function should work
        assert!(std::ptr::eq(&builder as *const _, &builder as *const _));
    }
}

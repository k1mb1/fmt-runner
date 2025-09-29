use crate::cli::handler::handle_cli;
use crate::parser::LanguageProvider;
use crate::pipeline::{Pass, Pipeline};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// Builder for CLI runner that allows configuring the language provider and pipeline
///
/// This builder provides a fluent interface for setting up and running the CLI
/// with specific language providers and passes. You can add passes one by one
/// using the `add_pass` method, which is the recommended approach.
///
/// # Type Parameters
/// * `Language` - A type that implements `LanguageProvider` for language-specific operations
/// * `Config` - Configuration type that can be serialized/deserialized
///
/// # Examples
///
/// Basic usage with individual passes:
/// ```rust
/// use fmt_runner::{cli_builder, Pass};
///
/// cli_builder::<MyLanguage, MyConfig>()
///     .add_pass(IndentationPass)
///     .add_pass(LineLengthPass)
///     .run();
/// ```
///
/// Advanced usage with custom configuration:
/// ```rust
/// cli_builder::<MyLanguage, MyConfig>()
///     .add_pass(MyPass)
///     .advanced()
///     .with_binary_name("my-formatter")
///     .run();
/// ```
pub struct CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    pipeline: Pipeline<Config>,
    _language_marker: PhantomData<Language>,
}

/// Advanced CLI builder with additional configuration options
///
/// This builder extends the basic `CliBuilder` with additional configuration
/// capabilities for more complex CLI setups.
pub struct AdvancedCliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    builder: CliBuilder<Language, Config>,
    custom_binary_name: Option<String>,
    custom_config_name: Option<String>,
}

impl<Language, Config> CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    /// Create a new CLI builder instance
    ///
    /// # Returns
    /// A new `CliBuilder` instance ready for configuration
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            _language_marker: PhantomData,
        }
    }

    /// Add a pass to the formatting pipeline
    ///
    /// # Arguments
    /// * `pass` - The pass to add to the pipeline
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn add_pass<P>(mut self, pass: P) -> Self
    where
        P: Pass<Config = Config> + 'static,
    {
        self.pipeline.add_pass(pass);
        self
    }

    /// Set the pipeline to use for formatting operations (for backward compatibility)
    ///
    /// # Arguments
    /// * `pipeline` - The formatting pipeline to use
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn with_pipeline(mut self, pipeline: Pipeline<Config>) -> Self {
        self.pipeline = pipeline;
        self
    }

    /// Convert to an advanced builder for additional configuration options
    ///
    /// # Returns
    /// An `AdvancedCliBuilder` instance with the current configuration
    pub fn advanced(self) -> AdvancedCliBuilder<Language, Config> {
        AdvancedCliBuilder {
            builder: self,
            custom_binary_name: None,
            custom_config_name: None,
        }
    }

    /// Build and run the CLI with the configured settings
    ///
    /// This method will parse command line arguments and execute the appropriate command
    /// (init or format) based on the provided input.
    ///
    /// # Errors
    /// This function will print error messages to stderr and call `process::exit(1)`
    /// if any critical error occurs during CLI processing.
    pub fn run(self) {
        handle_cli::<Language, Config>(self.pipeline);
    }

    /// Try to build and run the CLI
    ///
    /// This method consumes the builder and runs the CLI with the configured pipeline.
    ///
    /// # Errors
    /// This function will print error messages to stderr and call `process::exit(1)`
    /// if any critical error occurs during CLI processing.
    pub fn try_run(self) {
        handle_cli::<Language, Config>(self.pipeline);
    }
}

impl<Language, Config> AdvancedCliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    /// Set a custom binary name for the CLI
    ///
    /// # Arguments
    /// * `name` - The custom binary name to use
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn with_binary_name<S: Into<String>>(mut self, name: S) -> Self {
        self.custom_binary_name = Some(name.into());
        self
    }

    /// Set a custom config file name
    ///
    /// # Arguments
    /// * `name` - The custom config file name to use
    ///
    /// # Returns
    /// The builder instance for method chaining
    pub fn with_config_name<S: Into<String>>(mut self, name: S) -> Self {
        self.custom_config_name = Some(name.into());
        self
    }

    /// Convert back to a basic builder
    ///
    /// # Returns
    /// The underlying `CliBuilder` instance
    pub fn basic(self) -> CliBuilder<Language, Config> {
        self.builder
    }

    /// Build and run the CLI with the configured settings
    ///
    /// # Panics
    /// Panics if no pipeline has been configured using `with_pipeline()`
    ///
    /// # Errors
    /// This function will print error messages to stderr and call `process::exit(1)`
    /// if any critical error occurs during CLI processing.
    pub fn run(self) {
        // For now, we'll use the basic implementation
        // In a full implementation, you'd use custom_binary_name and custom_config_name
        self.builder.run();
    }

    /// Try to build and run the CLI
    ///
    /// # Panics
    /// Panics if no pipeline has been configured using `with_pipeline()`
    ///
    /// # Errors
    /// This function will print error messages to stderr and call `process::exit(1)`
    /// if any critical error occurs during CLI processing.
    pub fn try_run(self) {
        self.builder.try_run();
    }
}

impl<Language, Config> Default for CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a new CLI builder
///
/// # Type Parameters
/// * `Language` - A type that implements `LanguageProvider` for language-specific operations
/// * `Config` - Configuration type that can be serialized/deserialized
///
/// # Returns
/// A new `CliBuilder` instance with an empty pipeline
///
/// # Example
/// ```rust
/// use fmt_runner::{cli_builder, Pass};
///
/// cli_builder::<MyLanguage, MyConfig>()
///     .add_pass(IndentationPass)
///     .add_pass(LineLengthPass)
///     .run();
/// ```
pub fn cli_builder<Language, Config>() -> CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    CliBuilder::new()
}

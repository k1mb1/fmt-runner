use crate::cli::handler::handle_cli;
use crate::parser::LanguageProvider;
use crate::pipeline::{Pass, Pipeline};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

/// Builder for CLI runner with fluent interface
///
/// Add passes one by one using `add_pass` method
pub struct CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    pipeline: Pipeline<Config>,
    _language_marker: PhantomData<Language>,
}

impl<Language, Config> CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    /// Create new CLI builder
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            _language_marker: PhantomData,
        }
    }

    /// Add pass to the pipeline
    pub fn add_pass<P>(mut self, pass: P) -> Self
    where
        P: Pass<Config = Config> + 'static,
    {
        self.pipeline.add_pass(pass);
        self
    }

    /// Set the pipeline to use
    pub fn with_pipeline(mut self, pipeline: Pipeline<Config>) -> Self {
        self.pipeline = pipeline;
        self
    }

    /// Run the CLI
    pub fn run(self) {
        handle_cli::<Language, Config>(self.pipeline);
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

/// Create new CLI builder
pub fn cli_builder<Language, Config>() -> CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    CliBuilder::new()
}

use crate::cli::handler::handle_cli;
use crate::core::ConfigProvider;
use crate::parser::LanguageProvider;
use crate::pipeline::{Pass, Pipeline};
use std::marker::PhantomData;

pub struct CliBuilder<Language: LanguageProvider, Config: ConfigProvider> {
    pipeline: Pipeline<Config>,
    _language_marker: PhantomData<Language>,
}

impl<Language: LanguageProvider, Config: ConfigProvider> CliBuilder<Language, Config> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            _language_marker: PhantomData,
        }
    }

    #[must_use]
    pub fn add_pass<P: Pass<Config = Config> + 'static>(mut self, pass: P) -> Self {
        self.pipeline.add_pass(pass);
        self
    }

    #[must_use]
    pub fn with_pipeline(mut self, pipeline: Pipeline<Config>) -> Self {
        self.pipeline = pipeline;
        self
    }

    pub fn run(self) {
        handle_cli::<Language, Config>(self.pipeline);
    }
}

impl<Language: LanguageProvider, Config: ConfigProvider> Default for CliBuilder<Language, Config> {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn cli_builder<Language: LanguageProvider, Config: ConfigProvider>(
) -> CliBuilder<Language, Config> {
    CliBuilder::new()
}

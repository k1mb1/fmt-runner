use crate::cli::handler::handle_cli;
use crate::parser::LanguageProvider;
use crate::pipeline::{Pass, Pipeline};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

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
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
            _language_marker: PhantomData,
        }
    }

    #[must_use]
    pub fn add_pass<P>(mut self, pass: P) -> Self
    where
        P: Pass<Config = Config> + 'static,
    {
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

impl<Language, Config> Default for CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn cli_builder<Language, Config>() -> CliBuilder<Language, Config>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    CliBuilder::new()
}

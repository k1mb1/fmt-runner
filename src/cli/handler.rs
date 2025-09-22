use crate::cli::cli_entry::{Cli, Commands};
use crate::cli::commands::{format, init};
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use clap::Parser;
use serde::{de::DeserializeOwned, Serialize};


pub fn handle_cli<Language, Config>(pipeline: Pipeline<Config>)
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let cli = Cli::parse();
    match cli.commands {
        Commands::Init { config_path } => init::<Config>(config_path),
        Commands::Format {
            config_path,
            files_path,
        } => format::<Language, Config>(config_path, files_path, pipeline),
    }
}

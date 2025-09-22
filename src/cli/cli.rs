use clap::{Parser, Subcommand};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

use crate::cli::commands::{format, init};
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;


pub const DEFAULT_CONFIG_FILE: &'static str = "jvfmt.yml";

pub fn handle_cli<L, C>(pipeline: Pipeline<C>)
where
    C: Serialize + DeserializeOwned + Default,
    L: LanguageProvider,
{
    let cli = Cli::parse();
    match cli.commands {
        Commands::Init { config_path } => init::<C>(config_path),
        Commands::Format {
            config_path,
            files_path,
        } => format::<L, C>(config_path, files_path, pipeline),
    }
}

#[derive(Parser, Debug)]
#[command(about = "formatter tool")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new configuration file
    Init {
        /// Create configuration file with a given name, or validate an existing one for errors
        #[arg(
            short = 'f',
            long = "file",
            value_name = "FILENAME",
            default_value = DEFAULT_CONFIG_FILE
        )]
        config_path: PathBuf,
    },

    /// Форматирует указанные файлы
    Format {
        #[arg(
            short = 'c',
            long = "config",
            value_name = "FILENAME",
            default_value = DEFAULT_CONFIG_FILE
        )]
        config_path: PathBuf,

        #[arg(value_name = "FILES", default_value = ".")]
        files_path: Vec<PathBuf>,
        //TODO сделать вывод просто check или в файл
    },
}

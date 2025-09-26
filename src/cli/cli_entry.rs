use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub const DEFAULT_CONFIG_FILE: &str = concat!(env!("CARGO_PKG_NAME"), ".yml"); //TODO сделать чтобы bin name был а не библиотеки

#[derive(Parser, Debug)]
#[command(about = "formatter tool")]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new configuration file
    Init {
        /// Create configuration file with a given name, or validate an existing one for errors
        #[arg(
            short = 'c',
            long = "config",
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

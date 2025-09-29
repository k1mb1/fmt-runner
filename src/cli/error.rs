use thiserror::Error;

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Config path argument is missing")]
    ConfigPathMissing,

    #[error("Files path argument is missing")]
    FilesPathMissing,

    #[error("No valid subcommand provided. Use --help for usage information")]
    NoValidSubcommand,

    #[error("Unknown command '{command}'")]
    UnknownCommand { command: String },

    #[error("Failed to get binary name from command line arguments")]
    BinaryNameError,

    #[error("Config file has unsupported extension")]
    UnsupportedConfigExtension,

    #[error("YAML parsing error: {source}")]
    YamlError {
        #[from]
        source: serde_yaml::Error,
    },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// Exit the program with a CLI error
///
/// This function prints the error message to stderr and exits the program
/// with status code 1. It's intended for fatal errors that should terminate
/// the application immediately.
pub fn exit_with_error(error: CliError) -> ! {
    eprintln!("Error: {}", error);
    std::process::exit(1);
}

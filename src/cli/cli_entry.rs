use clap::{Arg, Command};

/// Available CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommand {
    /// Initialize a new configuration file
    Init,
    /// Format source files and write changes
    Format,
    /// Check if files are formatted correctly
    Check,
}

impl CliCommand {
    const INIT: &'static str = "init";
    const FORMAT: &'static str = "format";
    const CHECK: &'static str = "check";

    /// Get the string representation of the CLI command.
    pub fn as_str(self) -> &'static str {
        match self {
            CliCommand::Init => Self::INIT,
            CliCommand::Format => Self::FORMAT,
            CliCommand::Check => Self::CHECK,
        }
    }
}

/// Get config filename by binary name.
///
/// # Arguments
/// * `bin_name` - The name of the binary
///
/// # Returns
/// The default configuration filename (e.g., "mybin.yml")
fn default_config_name(bin_name: &str) -> String {
    format!("{bin_name}.yml")
}

/// Create a config argument with a default value.
///
/// # Arguments
/// * `default` - The default config filename
fn config_arg(default: &'static str) -> Arg {
    Arg::new("config_path")
        .short('c')
        .long("config")
        .value_name("FILENAME")
        .default_value(default)
        .help("Path to the configuration file")
}

/// Build CLI with dynamic binary and config names.
///
/// # Arguments
/// * `bin_name` - The name of the binary (used for help text and defaults)
///
/// # Returns
/// A configured `Command` ready to parse arguments
pub fn build_cli(bin_name: &str) -> Command {
    let bin_name_leaked: &'static str = Box::leak(bin_name.to_string().into_boxed_str());
    let config_leaked: &'static str = Box::leak(default_config_name(bin_name).into_boxed_str());

    Command::new(bin_name_leaked)
        .about("Formatter tool")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new(CliCommand::Init.as_str())
                .about("Create a new configuration file")
                .arg(config_arg(config_leaked)),
        )
        .subcommand(
            Command::new(CliCommand::Format.as_str())
                .about("Format files and write changes to disk")
                .arg(config_arg(config_leaked))
                .arg(
                    Arg::new("files_path")
                        .value_name("FILES")
                        .default_value(".")
                        .num_args(1..)
                        .help("Files or directories to format"),
                ),
        )
        .subcommand(
            Command::new(CliCommand::Check.as_str())
                .about("Check if files are formatted correctly")
                .arg(config_arg(config_leaked))
                .arg(
                    Arg::new("files_path")
                        .value_name("FILES")
                        .default_value(".")
                        .num_args(1..)
                        .help("Files or directories to check"),
                )
                .arg(
                    Arg::new("diff")
                        .short('d')
                        .long("diff")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show differences for files that need formatting"),
                ),
        )
}

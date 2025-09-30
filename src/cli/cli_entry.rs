use clap::{Arg, Command};

/// Format modes for the formatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    /// Check if files are formatted without writing changes
    Check,
    /// Format files and write changes to disk
    Write,
    /// Show a unified diff of the changes without modifying files
    Diff,
}

impl FormatMode {
    const CHECK: &'static str = "check";
    const WRITE: &'static str = "write";
    const DIFF: &'static str = "diff";

    /// Get the string representation of the format mode.
    pub fn as_str(self) -> &'static str {
        match self {
            FormatMode::Check => Self::CHECK,
            FormatMode::Write => Self::WRITE,
            FormatMode::Diff => Self::DIFF,
        }
    }
}

/// Available CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommand {
    /// Initialize a new configuration file
    Init,
    /// Format source files
    Format,
}

impl CliCommand {
    const INIT: &'static str = "init";
    const FORMAT: &'static str = "format";

    /// Get the string representation of the CLI command.
    pub fn as_str(self) -> &'static str {
        match self {
            CliCommand::Init => Self::INIT,
            CliCommand::Format => Self::FORMAT,
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
                .about("Format specified files")
                .arg(config_arg(config_leaked))
                .arg(
                    Arg::new("files_path")
                        .value_name("FILES")
                        .default_value(".")
                        .num_args(1..)
                        .help("Files or directories to format"),
                )
                .arg(
                    Arg::new("mode")
                        .short('m')
                        .long("mode")
                        .value_name("MODE")
                        .default_value(FormatMode::Check.as_str())
                        .value_parser([
                            FormatMode::Check.as_str(),
                            FormatMode::Write.as_str(),
                            FormatMode::Diff.as_str(),
                        ])
                        .help(format!(
                            "Format mode: '{}' to only verify formatting, '{}' to apply changes, '{}' to print a unified diff",
                            FormatMode::Check.as_str(),
                            FormatMode::Write.as_str(),
                            FormatMode::Diff.as_str()
                        )),
                ),
        )
}

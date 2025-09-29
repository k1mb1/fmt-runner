use clap::{Arg, Command};

/// CLI commands
#[derive(Debug, Clone, Copy)]
pub enum CliCommand {
    Init,
    Format,
}

impl CliCommand {
    const INIT: &'static str = "init";
    const FORMAT: &'static str = "format";

    pub fn as_str(&self) -> &'static str {
        match self {
            CliCommand::Init => Self::INIT,
            CliCommand::Format => Self::FORMAT,
        }
    }
}

/// Get config filename by binary name
fn default_config_name(bin_name: &str) -> String {
    format!("{bin_name}.yml")
}

fn config_arg(default: &'static str) -> Arg {
    Arg::new("config_path")
        .short('c')
        .long("config")
        .value_name("FILENAME")
        .default_value(default)
}

/// Build CLI with dynamic binary and config names
pub fn build_cli(bin_name: &str) -> Command {
    let bin_name_leaked: &'static str = Box::leak(bin_name.to_string().into_boxed_str());
    let config_leaked: &'static str = Box::leak(default_config_name(bin_name).into_boxed_str());

    Command::new(bin_name_leaked)
        .about("Formatter tool")
        .version(env!("CARGO_PKG_VERSION")) //TODO use clap's built-in version feature or not?
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
                        .num_args(1..),
                ),
        )
}

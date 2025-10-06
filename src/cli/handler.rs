use crate::cli::cli_entry::{build_cli, CliCommand};
use crate::cli::commands::{check, format, init};
use crate::cli::error::{exit_with_error, CliError, CliResult};
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use serde::{de::DeserializeOwned, Serialize};
use std::env;
use std::path::{Path, PathBuf};

fn parse_command(cmd_str: &str) -> Option<CliCommand> {
    match cmd_str {
        cmd if cmd == CliCommand::Init.as_str() => Some(CliCommand::Init),
        cmd if cmd == CliCommand::Format.as_str() => Some(CliCommand::Format),
        cmd if cmd == CliCommand::Check.as_str() => Some(CliCommand::Check),
        _ => None,
    }
}

pub fn handle_cli<Language, Config>(pipeline: Pipeline<Config>)
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    if let Err(e) = try_handle_cli::<Language, Config>(pipeline) {
        exit_with_error(&e);
    }
}

fn try_handle_cli<Language, Config>(pipeline: Pipeline<Config>) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let bin_name = get_binary_name().unwrap_or_else(|_| "fmt-runner".to_string());
    let matches = build_cli(&bin_name).get_matches();

    match matches.subcommand() {
        Some((cmd_str, sub_matches)) => match parse_command(cmd_str) {
            Some(CliCommand::Init) => {
                handle_init_command::<Config>(sub_matches)?;
            }
            Some(CliCommand::Format) => {
                handle_format_command::<Language, Config>(sub_matches, pipeline)?;
            }
            Some(CliCommand::Check) => {
                handle_check_command::<Language, Config>(sub_matches, pipeline)?;
            }
            None => {
                exit_with_error(&CliError::UnknownCommand {
                    command: cmd_str.to_string(),
                });
            }
        },
        None => {
            exit_with_error(&CliError::NoValidSubcommand);
        }
    }

    Ok(())
}

fn get_binary_name() -> CliResult<String> {
    env::args()
        .next()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(std::string::ToString::to_string)
        })
        .ok_or(CliError::BinaryNameError)
}

fn handle_init_command<Config>(sub_matches: &clap::ArgMatches) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
{
    let config_path = sub_matches
        .get_one::<String>("config_path")
        .ok_or(CliError::ConfigPathMissing)?;

    init::<Config>(config_path.into())?;
    Ok(())
}

fn handle_format_command<Language, Config>(
    sub_matches: &clap::ArgMatches,
    pipeline: Pipeline<Config>,
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config_path = sub_matches
        .get_one::<String>("config_path")
        .ok_or(CliError::ConfigPathMissing)?;

    let files_path: Vec<String> = sub_matches
        .get_many::<String>("files_path")
        .ok_or(CliError::FilesPathMissing)?
        .cloned()
        .collect();

    let files_path: Vec<PathBuf> = files_path.into_iter().map(PathBuf::from).collect();

    format::<Language, Config>(Path::new(config_path), &files_path, pipeline)?;

    Ok(())
}

fn handle_check_command<Language, Config>(
    sub_matches: &clap::ArgMatches,
    pipeline: Pipeline<Config>,
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config_path = sub_matches
        .get_one::<String>("config_path")
        .ok_or(CliError::ConfigPathMissing)?;

    let files_path: Vec<String> = sub_matches
        .get_many::<String>("files_path")
        .ok_or(CliError::FilesPathMissing)?
        .cloned()
        .collect();

    let show_diff = sub_matches.get_flag("diff");

    let files_path: Vec<PathBuf> = files_path.into_iter().map(PathBuf::from).collect();

    check::<Language, Config>(Path::new(config_path), &files_path, pipeline, show_diff)?;

    Ok(())
}

use crate::cli::commands::utils::init_config;
use crate::cli::error::CliResult;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub fn execute<Config>(config_path: PathBuf) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
{
    //TODO validate config
    //TODO if validate what is value missing

    init_config::<Config>(config_path.as_path())?;

    println!(
        "Configuration file created successfully at: {}",
        config_path.display()
    );
    Ok(())
}

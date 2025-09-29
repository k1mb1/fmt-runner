use crate::cli::commands::utils::{
    check_extension, create_default_file, exists_config, validate_config,
};
use crate::cli::error::{CliError, CliResult};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub fn execute<Config>(config_path: PathBuf) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
{
    if exists_config(&config_path)? {
        println!("Config file already exists, skipping creation.");
        validate_config::<Config>(&config_path)?;
    } else {
        check_extension(&config_path)?;
        create_default_file::<Config>(&config_path)?;
    }

    println!(
        "Configuration file created successfully at: {}",
        config_path.display()
    );
    Ok(())
}

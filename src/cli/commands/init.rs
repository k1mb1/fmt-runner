use crate::cli::commands::utils::{
    check_extension, create_default_file, exists_config, validate_config,
};
use crate::cli::error::CliResult;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub fn execute<Config>(config_path: PathBuf) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
{
    if exists_config(&config_path)? {
        println!("Config file already exists, validating...");
        validate_config::<Config>(&config_path)?;
        println!("Config at {} is valid.", config_path.display());
    } else {
        check_extension(&config_path)?;
        println!(
            "Config file not found. Creating default at {}...",
            config_path.display()
        );
        create_default_file::<Config>(&config_path)?;
        println!("Default configuration created at {}", config_path.display());
    }

    // final confirmation
    println!("Configuration available at: {}", config_path.display());
    Ok(())
}

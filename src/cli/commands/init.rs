use crate::cli::commands::ConfigLoader;
use crate::cli::error::CliResult;
use log::info;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

/// Execute the init command to create or validate a configuration file.
///
/// This function uses ConfigLoader to:
/// 1. Check if config file exists and validate it
/// 2. Create a default config file if it doesn't exist
///
/// # Arguments
/// * `config_path` - Path where the config file should be created or validated
///
/// # Returns
/// `Ok(())` on success, or an error if validation or creation fails
pub fn execute<Config>(config_path: PathBuf) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
{
    if ConfigLoader::exists(&config_path)? {
        info!("Config file already exists, validating...");
        ConfigLoader::validate::<Config>(&config_path)?;
        info!("✓ Config at {} is valid.", config_path.display());
    } else {
        ConfigLoader::check_extension(&config_path)?;
        info!(
            "Config file not found. Creating default at {}...",
            config_path.display()
        );
        ConfigLoader::create_default_file::<Config>(&config_path)?;
        info!(
            "✓ Default configuration created at {}",
            config_path.display()
        );
    }

    info!("✓ Configuration available at: {}", config_path.display());
    Ok(())
}

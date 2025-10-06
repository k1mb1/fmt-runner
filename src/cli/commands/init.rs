use crate::cli::commands::ConfigLoader;
use crate::cli::error::CliResult;
use crate::core::ConfigProvider;
use log::info;
use std::path::PathBuf;

/// Execute the init command to create or validate a configuration file.
pub fn execute<Config: ConfigProvider>(config_path: PathBuf) -> CliResult<()> {
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

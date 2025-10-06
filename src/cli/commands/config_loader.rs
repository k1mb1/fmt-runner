use crate::cli::error::{CliError, CliResult};
use crate::supported_extension::CONFIG_EXTENSIONS;
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Configuration loader responsible for loading and validating config files.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load config or create default when missing.
    ///
    /// # Arguments
    /// * `config_path` - Path to the config file
    ///
    /// # Returns
    /// The loaded or default config
    pub fn load<Config>(config_path: &Path) -> CliResult<Config>
    where
        Config: Serialize + DeserializeOwned + Default,
    {
        info!("Loading config from {}...", config_path.display());

        let config = if Self::exists(config_path)? {
            Self::validate_config::<Config>(config_path)?;
            Self::from_file(config_path)?
        } else {
            Self::check_extension(config_path)?;
            debug!(
                "Config file not found, creating default at {}...",
                config_path.display()
            );
            Config::default()
        };

        Ok(config)
    }

    /// Write a default config file (creates parent directories if needed).
    ///
    /// # Arguments
    /// * `path` - Path where the config file should be created
    ///
    /// # Returns
    /// `Ok(())` on success, or an error
    pub fn create_default_file<Config: Serialize + Default>(path: &Path) -> CliResult<()> {
        let default_config = Config::default();
        let yaml = serde_yaml::to_string(&default_config)?;

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(path, yaml)?;
        Ok(())
    }

    /// Check if a valid config file exists at the given path.
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// `Ok(true)` if valid config exists, `Ok(false)` if not, error if path is invalid
    pub fn exists(path: &Path) -> CliResult<bool> {
        if path.exists() {
            if path.is_dir() {
                return Err(CliError::ConfigPathIsDirectory);
            }
            Self::check_extension(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Validate config file by attempting to load it.
    ///
    /// # Arguments
    /// * `path` - Path to the config file
    ///
    /// # Returns
    /// `Ok(())` if config is valid, error otherwise
    pub fn validate<Config>(path: &Path) -> CliResult<()>
    where
        Config: Serialize + DeserializeOwned + Default,
    {
        Self::load::<Config>(path)?;
        Ok(())
    }

    /// Check if the config file path has a supported extension.
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// `Ok(())` if extension is supported, error otherwise
    pub fn check_extension(path: &Path) -> CliResult<()> {
        if !CONFIG_EXTENSIONS.matches(path) {
            return Err(CliError::UnsupportedConfigExtension);
        }
        Ok(())
    }

    /// Deserialize a config from YAML string.
    ///
    /// # Arguments
    /// * `yaml` - YAML string to deserialize
    ///
    /// # Returns
    /// The deserialized config or a YAML error
    fn from_str<Config: DeserializeOwned>(yaml: &str) -> CliResult<Config> {
        serde_yaml::from_str(yaml).map_err(CliError::from)
    }

    /// Load config from a file path.
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    /// The loaded config or an error
    fn from_file<Config: DeserializeOwned>(config_path: &Path) -> CliResult<Config> {
        let config_content = fs::read_to_string(config_path)?;
        Self::from_str(&config_content)
    }

    /// Validate config content by deserializing it (private helper).
    ///
    /// # Arguments
    /// * `path` - Path to the config file
    ///
    /// # Returns
    /// `Ok(())` if config is valid, error otherwise
    fn validate_config<Config: DeserializeOwned>(path: &Path) -> CliResult<()> {
        Self::from_file::<Config>(path)?;
        Ok(())
    }
}

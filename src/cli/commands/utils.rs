use std::collections::HashSet;

use crate::cli::error::{CliError, CliResult};
use crate::parser::LanguageProvider;
use crate::supported_extension::{SupportedExtension, CONFIG_EXTENSIONS};
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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
    from_str(&config_content)
}

/// Write a default config file (creates parent directories if needed).
///
/// # Arguments
/// * `path` - Path where the config file should be created
///
/// # Returns
/// `Ok(())` on success, or an error
pub(crate) fn create_default_file<Config: Serialize + Default>(path: &Path) -> CliResult<()> {
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

/// Ensure config file has supported extension.
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `Ok(())` if extension is supported, error otherwise
pub(crate) fn check_extension(path: &Path) -> CliResult<()> {
    if !CONFIG_EXTENSIONS.matches(path) {
        return Err(CliError::UnsupportedConfigExtension);
    }
    Ok(())
}

/// Check whether a valid config file exists at path.
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `Ok(true)` if valid config exists, `Ok(false)` if not, error if path is invalid
pub(crate) fn exists_config(path: &Path) -> CliResult<bool> {
    if path.exists() {
        if path.is_dir() {
            return Err(CliError::ConfigPathIsDirectory);
        }
        check_extension(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Validate config content by deserializing it.
///
/// # Arguments
/// * `path` - Path to the config file
///
/// # Returns
/// `Ok(())` if config is valid, error otherwise
pub(crate) fn validate_config<Config: DeserializeOwned>(path: &Path) -> CliResult<()> {
    from_file::<Config>(path)?;
    Ok(())
}

/// Load config or create default when missing.
///
/// # Arguments
/// * `config_path` - Path to the config file
///
/// # Returns
/// The loaded or default config
pub(crate) fn load_config<Config>(config_path: &Path) -> CliResult<Config>
where
    Config: Serialize + DeserializeOwned + Default,
{
    info!("Loading config from {}...", config_path.display());

    let config = if exists_config(config_path)? {
        validate_config::<Config>(config_path)?;
        from_file(config_path)?
    } else {
        check_extension(config_path)?;
        debug!(
            "Config file not found, creating default at {}...",
            config_path.display()
        );
        Config::default()
    };

    Ok(config)
}

/// Collect supported files from path (file or directory).
///
/// # Arguments
/// * `root` - Root path to search from
///
/// # Returns
/// Vector of supported file paths
pub(crate) fn collect_supported_files<Language: LanguageProvider>(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let supported = Language::supported_extension();

    if root.is_file() {
        if supported.matches(root) {
            files.push(root.to_path_buf());
        }
    } else if root.is_dir() {
        collect_files_recursive(root, supported, &mut files);
    }

    files
}

/// Helper: recursively walk directory and push supported files.
fn collect_files_recursive(dir: &Path, supported: &SupportedExtension, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, supported, files);
            } else if supported.matches(&path) {
                files.push(path);
            }
        }
    }
}

/// Collect unique supported files from multiple paths.
///
/// This function deduplicates files and returns them in sorted order.
///
/// # Arguments
/// * `paths` - Array of paths to search
///
/// # Returns
/// Sorted vector of unique file paths
pub(crate) fn collect_all_supported_files<Language: LanguageProvider>(
    paths: &[PathBuf],
) -> Vec<PathBuf> {
    let mut files_set = HashSet::new();
    let mut files_vec = Vec::new();

    for path in paths {
        for file in collect_supported_files::<Language>(path) {
            if files_set.insert(file.clone()) {
                files_vec.push(file);
            }
        }
    }

    // Stable ordering: sort by display string
    files_vec.sort_by_key(|p| p.display().to_string());
    files_vec
}

/// Read given files into strings.
///
/// # Arguments
/// * `files` - Array of file paths to read
///
/// # Returns
/// Vector of file contents as strings, or first IO error encountered
pub(crate) fn read_files_to_strings(files: &[PathBuf]) -> CliResult<Vec<String>> {
    let mut contents = Vec::with_capacity(files.len());

    for file_path in files {
        let content = fs::read_to_string(file_path)?;
        contents.push(content);
    }

    Ok(contents)
}

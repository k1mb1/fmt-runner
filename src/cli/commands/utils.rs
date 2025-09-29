use std::collections::HashSet;

use crate::cli::error::{CliError, CliResult};
use crate::parser::LanguageProvider;
use crate::supported_extension::{SupportedExtension, CONFIG_EXTENSIONS};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Deserialize a config from YAML string.
fn from_str<Config: DeserializeOwned>(yaml: &str) -> CliResult<Config> {
    serde_yaml::from_str(yaml).map_err(CliError::from)
}

/// Load config from a file path.
fn from_file<Config: DeserializeOwned>(config_path: &Path) -> CliResult<Config> {
    let config_content = fs::read_to_string(config_path)?;
    from_str(&config_content)
}

/// Write a default config file (creates parents).
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
pub(crate) fn check_extension(path: &Path) -> CliResult<()> {
    if !CONFIG_EXTENSIONS.matches(path) {
        return Err(CliError::UnsupportedConfigExtension);
    }
    Ok(())
}

/// Check whether a valid config file exists at path.
pub(crate) fn exists_config(path: &Path) -> CliResult<bool> {
    if path.exists() {
        if path.is_dir() {
            // it's an error to point a config path at a directory
            return Err(crate::cli::error::CliError::ConfigPathIsDirectory);
        }
        check_extension(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Validate config content by deserializing it.
pub(crate) fn validate_config<Config: DeserializeOwned>(path: &Path) -> CliResult<()> {
    // TODO: add semantic/content validation
    from_file::<Config>(path)?;
    Ok(())
}

/// Load config or create default when missing.
pub(crate) fn load_config<Config>(config_path: &Path) -> CliResult<Config>
where
    Config: Serialize + DeserializeOwned + Default,
{
    println!("Loading config from {}...", config_path.display());
    let config = if exists_config(config_path)? {
        validate_config::<Config>(config_path)?;
        from_file(config_path)?
    } else {
        check_extension(config_path)?;
        println!(
            "Config file not found, creating default at {}...",
            config_path.display()
        );
        Config::default()
    };
    Ok(config)
}

/// Collect supported files from path (file or directory).
pub(crate) fn collect_supported_files<Language: LanguageProvider>(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let supported = Language::supported_extension();
    if root.is_file() {
        if supported.matches(root) {
            files.push(root.to_path_buf());
        }
    } else {
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
    // stable ordering: sort by display string
    files_vec.sort_by_key(|p| p.display().to_string());
    files_vec
}

/// Read given files into strings (fails on first IO error).
/// For tolerant behavior use an alternative implementation.
pub(crate) fn read_files_to_strings(files: &[PathBuf]) -> CliResult<Vec<String>> {
    let mut contents = Vec::with_capacity(files.len());
    for file_path in files {
        let content = fs::read_to_string(file_path)?;
        contents.push(content);
    }
    Ok(contents)
}

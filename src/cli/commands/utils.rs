use std::collections::HashSet;

use crate::cli::error::{CliError, CliResult};
use crate::parser::LanguageProvider;
use crate::supported_extension::{SupportedExtension, CONFIG_EXTENSIONS};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Deserialize a Config from a YAML string.
fn from_str<Config: DeserializeOwned>(yaml: &str) -> CliResult<Config> {
    serde_yaml::from_str(yaml).map_err(CliError::from)
}

/// Load Config from file, returning a CliError on failure.
fn from_file<Config: DeserializeOwned>(config_path: &Path) -> CliResult<Config> {
    let config_content = fs::read_to_string(config_path)?;
    from_str(&config_content)
}

/// Create a default Config file at the given path.
pub(crate) fn create_default_file<Config: Serialize + Default>(path: &Path) -> CliResult<()> {
    let default_config = Config::default();
    let yaml = serde_yaml::to_string(&default_config)?;
    fs::write(path, yaml)?;
    Ok(())
}

pub(crate) fn check_extension(path: &Path) -> CliResult<()> {
    if !CONFIG_EXTENSIONS.matches(path) {
        return Err(CliError::UnsupportedConfigExtension);
    }
    Ok(())
}

pub(crate) fn exists_config(path: &Path) -> CliResult<bool> {
    if path.exists() {
        check_extension(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(crate) fn validate_config<Config: DeserializeOwned>(path: &Path) -> CliResult<()> {
    //TODO validate content
    from_file::<Config>(path)?;
    Ok(())
}

pub(crate) fn load_config<Config>(config_path: &Path) -> CliResult<Config>
where
    Config: Serialize + DeserializeOwned + Default,
{
    print!("Loading config from {}...\n", config_path.display());
    let config = if exists_config(&config_path)? {
        validate_config::<Config>(&config_path)?;
        from_file(config_path)?
    } else {
        check_extension(&config_path)?;
        print!("Config file not found, creating default at {}...\n", config_path.display());
        Config::default()
    };
    Ok(config)
}

/// Recursively collects all files in `root` and subdirectories with extensions supported by `L`.
pub(crate) fn collect_supported_files<Language: LanguageProvider>(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let supported = Language::supported_extension();
    collect_files_recursive(root, supported, &mut files);
    files
}

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

/// Collects unique files with supported extensions from a list of paths (files and directories).
pub(crate) fn collect_all_supported_files<Language: LanguageProvider>(
    paths: &[PathBuf],
) -> Vec<PathBuf> {
    let mut files_set = HashSet::new();
    for path in paths {
        for file in collect_supported_files::<Language>(path) {
            files_set.insert(file);
        }
    }
    files_set.into_iter().collect()
}

/// Reads the contents of files, returning an error if any file cannot be read.
/// For a version that skips unreadable files, use `read_files_to_strings_best_effort`.
pub(crate) fn read_files_to_strings(files: &[PathBuf]) -> CliResult<Vec<String>> {
    let mut contents = Vec::with_capacity(files.len());
    for file_path in files {
        let content = fs::read_to_string(file_path)?;
        contents.push(content);
    }
    Ok(contents)
}

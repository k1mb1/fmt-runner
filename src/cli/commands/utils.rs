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
fn create_default_file<Config: Serialize + Default>(path: &Path) -> CliResult<()> {
    let default_config = Config::default();
    let yaml = serde_yaml::to_string(&default_config)?;
    fs::write(path, yaml)?;
    Ok(())
}

/// Initialize config: if file does not exist, create with default config.
/// If file exists, validate its contents. Only files with supported extension allowed.
pub(crate) fn init_config<Config>(config_path: &Path) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
{
    if !CONFIG_EXTENSIONS.matches(config_path) {
        return Err(CliError::UnsupportedConfigExtension);
    }

    if config_path.exists() {
        println!("Config file already exists, validating...");
        from_file::<Config>(config_path)?;
    } else {
        create_default_file::<Config>(config_path)?;
        println!("Default config file created at {:?}", config_path);
    }
    Ok(())
}

/// Load config: if file exists, load; if not, return default.
/// Only files with supported extension allowed.
pub(crate) fn load_config<Config>(config_path: &Path) -> CliResult<Config>
where
    Config: Serialize + DeserializeOwned + Default,
{
    if config_path.exists() {
        if !CONFIG_EXTENSIONS.matches(config_path) {
            return Err(CliError::UnsupportedConfigExtension);
        }
        from_file::<Config>(config_path)
    } else {
        Ok(Config::default())
    }
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

/// Reads the contents of files, skipping those that cannot be read.
/// Returns the successfully read contents and a count of skipped files.
pub(crate) fn read_files_to_strings_best_effort(files: &[PathBuf]) -> (Vec<String>, usize) {
    let mut contents = Vec::new();
    let mut skipped = 0;

    for file_path in files {
        match fs::read_to_string(file_path) {
            Ok(content) => contents.push(content),
            Err(_) => {
                skipped += 1;
                eprintln!("Warning: Could not read file: {}", file_path.display());
            }
        }
    }

    (contents, skipped)
}

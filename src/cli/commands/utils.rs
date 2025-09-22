use std::collections::HashSet;

use crate::parser::LanguageProvider;
use crate::supported_extension::{SupportedExtension, CONFIG_EXTENSIONS};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};


/// Deserialize a Config from a YAML string.
fn from_str<C: DeserializeOwned>(yaml: &str) -> Result<C, serde_yaml::Error> {
    serde_yaml::from_str(yaml)
}

/// Load Config from file, returning a Boxed Error on failure.
fn from_file<C: DeserializeOwned>(config_path: &Path) -> Result<C, Box<dyn Error>> {
    let config_content = fs::read_to_string(config_path)?;
    from_str(&config_content).map_err(|e| e.into())
}

/// Create a default Config file at the given path.
fn create_default_file<C: Serialize + Default>(path: &Path) -> Result<(), Box<dyn Error>> {
    let default_config = C::default();
    let yaml = serde_yaml::to_string(&default_config)?;
    fs::write(path, yaml)?;
    Ok(())
}

/// Initialize config: if file does not exist, create with default config.
/// If file exists, validate its contents. Only files with supported extension allowed.
pub fn init_config<C>(config_path: &Path) -> Result<(), Box<dyn Error>>
where
    C: Serialize + DeserializeOwned + Default,
{
    if !CONFIG_EXTENSIONS.matches(config_path) {
        return Err("Config file has unsupported extension".into());
    }

    if config_path.exists() {
        println!("Config file already exists, validating...");
        from_file::<C>(config_path)?;
    } else {
        create_default_file::<C>(config_path)?;
        println!("Default config file created at {:?}", config_path);
    }
    Ok(())
}

/// Load config: if file exists, load; if not, return default.
/// Only files with supported extension allowed.
pub fn load_config<C>(config_path: &Path) -> Result<C, Box<dyn Error>>
where
    C: Serialize + DeserializeOwned + Default,
{
    if !CONFIG_EXTENSIONS.matches(config_path) {
        return Err("Config file has unsupported extension".into());
    }

    if config_path.exists() {
        from_file::<C>(config_path)
    } else {
        Ok(C::default())
    }
}

/// Recursively collects all files in `root` and subdirectories with extensions supported by `L`.
pub fn collect_supported_files<L: LanguageProvider>(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let supported = L::supported_extension();
    collect_files_recursive(root, &supported, &mut files);
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
pub fn collect_all_supported_files<L: LanguageProvider>(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files_set = HashSet::new();
    for path in paths {
        for file in collect_supported_files::<L>(path) {
            files_set.insert(file);
        }
    }
    files_set.into_iter().collect()
}

/// Reads the contents of files, skipping those that cannot be read.
pub fn read_files_to_strings(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect()
}

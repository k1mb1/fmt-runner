use crate::cli::error::{CliError, CliResult};
use crate::core::ConfigProvider;
use crate::supported_extension::CONFIG_EXTENSIONS;
use log::{debug, info};
use std::fs;
use std::path::Path;

/// Configuration loader responsible for loading and validating config files.
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load<Config: ConfigProvider>(config_path: &Path) -> CliResult<Config> {
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

    pub fn create_default_file<Config: ConfigProvider>(path: &Path) -> CliResult<()> {
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

    pub fn validate<Config: ConfigProvider>(path: &Path) -> CliResult<()> {
        Self::load::<Config>(path)?;
        Ok(())
    }

    pub fn check_extension(path: &Path) -> CliResult<()> {
        if !CONFIG_EXTENSIONS.matches(path) {
            return Err(CliError::UnsupportedConfigExtension);
        }
        Ok(())
    }

    fn from_str<Config: ConfigProvider>(yaml: &str) -> CliResult<Config> {
        serde_yaml::from_str(yaml).map_err(CliError::from)
    }

    fn from_file<Config: ConfigProvider>(config_path: &Path) -> CliResult<Config> {
        let config_content = fs::read_to_string(config_path)?;
        Self::from_str(&config_content)
    }

    fn validate_config<Config: ConfigProvider>(path: &Path) -> CliResult<()> {
        Self::from_file::<Config>(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use serde::{Deserialize, Serialize};
    use std::fs;
    use tempfile::TempDir;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
        enabled: bool,
    }

    impl TestConfig {
        fn new(name: &str, value: i32, enabled: bool) -> Self {
            Self {
                name: name.to_string(),
                value,
                enabled,
            }
        }
    }

    #[fixture]
    fn temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    fn config_path(temp_dir: &TempDir, filename: &str) -> std::path::PathBuf {
        temp_dir.path().join(filename)
    }

    #[rstest]
    fn test_load_existing_config(temp_dir: TempDir) {
        let path = config_path(&temp_dir, "config.yaml");
        let expected = TestConfig::new("test", 42, true);
        let yaml = serde_yaml::to_string(&expected).unwrap();
        fs::write(&path, yaml).unwrap();

        let loaded: TestConfig = ConfigLoader::load(&path).unwrap();
        assert_eq!(loaded, expected);
    }

    #[rstest]
    fn test_load_missing_config_creates_default(temp_dir: TempDir) {
        let path = config_path(&temp_dir, "missing.yaml");
        let config: TestConfig = ConfigLoader::load(&path).unwrap();
        assert_eq!(config, TestConfig::default());
    }

    #[rstest]
    fn test_load_invalid_yaml_returns_error(temp_dir: TempDir) {
        let path = config_path(&temp_dir, "invalid.yaml");
        fs::write(&path, "invalid: yaml: content: [").unwrap();

        let result = ConfigLoader::load::<TestConfig>(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::YamlError { .. }));
    }

    #[rstest]
    fn test_create_default_file(temp_dir: TempDir) {
        let path = config_path(&temp_dir, "new_config.yaml");
        ConfigLoader::create_default_file::<TestConfig>(&path).unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let loaded: TestConfig = serde_yaml::from_str(&content).unwrap();
        assert_eq!(loaded, TestConfig::default());
    }

    #[rstest]
    fn test_create_default_file_creates_parent_dirs(temp_dir: TempDir) {
        let path = temp_dir
            .path()
            .join("nested")
            .join("dirs")
            .join("config.yaml");
        ConfigLoader::create_default_file::<TestConfig>(&path).unwrap();

        assert!(path.exists());
        assert!(path.parent().unwrap().exists());
    }

    #[rstest]
    #[case("config.yaml", true)]
    #[case("config.yml", true)]
    #[case("nonexistent.yaml", false)]
    fn test_exists(temp_dir: TempDir, #[case] filename: &str, #[case] should_exist: bool) {
        let path = config_path(&temp_dir, filename);

        if should_exist {
            fs::write(&path, "name: test\nvalue: 0\nenabled: false").unwrap();
        }

        let result = ConfigLoader::exists(&path).unwrap();
        assert_eq!(result, should_exist);
    }

    #[rstest]
    fn test_exists_returns_error_for_directory(temp_dir: TempDir) {
        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();

        let result = ConfigLoader::exists(&dir_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CliError::ConfigPathIsDirectory
        ));
    }

    #[rstest]
    #[case("config.yaml")]
    #[case("config.yml")]
    #[case("CONFIG.YAML")]
    #[case("Config.YML")]
    fn test_check_extension_valid(#[case] filename: &str) {
        let path = Path::new(filename);
        let result = ConfigLoader::check_extension(path);
        assert!(result.is_ok());
    }

    #[rstest]
    #[case("config.txt")]
    #[case("config.json")]
    #[case("config")]
    #[case("config.toml")]
    fn test_check_extension_invalid(#[case] filename: &str) {
        let path = Path::new(filename);
        let result = ConfigLoader::check_extension(path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CliError::UnsupportedConfigExtension
        ));
    }

    #[rstest]
    fn test_validate_valid_config(temp_dir: TempDir) {
        let path = config_path(&temp_dir, "valid.yaml");
        let config = TestConfig::new("valid", 100, false);
        let yaml = serde_yaml::to_string(&config).unwrap();
        fs::write(&path, yaml).unwrap();

        let result = ConfigLoader::validate::<TestConfig>(&path);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_validate_invalid_config(temp_dir: TempDir) {
        let path = config_path(&temp_dir, "invalid.yaml");
        fs::write(&path, "name: test\nvalue: not_a_number\n").unwrap();

        let result = ConfigLoader::validate::<TestConfig>(&path);
        assert!(result.is_err());
    }

    #[rstest]
    fn test_load_with_nested_structure(temp_dir: TempDir) {
        #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
        struct NestedConfig {
            outer: String,
            inner: InnerConfig,
        }

        #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
        struct InnerConfig {
            field: i32,
        }

        let path = config_path(&temp_dir, "nested.yaml");
        let yaml = "outer: test\ninner:\n  field: 42\n";
        fs::write(&path, yaml).unwrap();

        let loaded: NestedConfig = ConfigLoader::load(&path).unwrap();
        assert_eq!(loaded.outer, "test");
        assert_eq!(loaded.inner.field, 42);
    }
}

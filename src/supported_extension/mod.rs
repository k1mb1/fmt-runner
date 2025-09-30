use std::path::Path;

pub static CONFIG_EXTENSIONS: SupportedExtension = SupportedExtension::new(&["yml", "yaml"]);

/// A wrapper type for a collection of supported file extensions.
#[derive(Debug)]
pub struct SupportedExtension {
    extensions: &'static [&'static str],
}

impl SupportedExtension {
    /// Creates a new instance with the given extensions (should be in lower case, without dots).
    pub const fn new(extensions: &'static [&'static str]) -> Self {
        Self { extensions }
    }

    /// Returns true if the given extension (case-insensitive, without dot) is supported.
    ///
    /// This is a private helper method used by the public `matches` method.
    fn contains(&self, extension: &str) -> bool {
        let ext = extension.to_ascii_lowercase();
        self.extensions.iter().any(|&e| e == ext)
    }

    /// Returns true if the path's extension matches one of this set (case-insensitive).
    pub fn matches(&self, path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => self.contains(ext),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const SUPPORTED: SupportedExtension = SupportedExtension::new(&["rs", "toml", "md"]);

    #[rstest]
    #[case("rs", true)]
    #[case("RS", true)]
    #[case("Md", true)]
    #[case("toml", true)]
    #[case("TOML", true)]
    #[case("exe", false)]
    #[case("txt", false)]
    fn test_contains(#[case] extension: &str, #[case] expected: bool) {
        assert_eq!(SUPPORTED.contains(extension), expected);
    }

    #[rstest]
    #[case("foo.rs", true)]
    #[case("bar.TOML", true)]
    #[case("/baz/qux.md", true)]
    #[case("test.MD", true)]
    #[case("noext", false)]
    #[case("foo.exe", false)]
    #[case("test.txt", false)]
    fn test_matches(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(SUPPORTED.matches(Path::new(path)), expected);
    }

    #[test]
    fn test_config_extensions() {
        assert!(CONFIG_EXTENSIONS.matches(Path::new("config.yml")));
        assert!(CONFIG_EXTENSIONS.matches(Path::new("config.yaml")));
        assert!(CONFIG_EXTENSIONS.matches(Path::new("config.YML")));
        assert!(CONFIG_EXTENSIONS.matches(Path::new("config.YAML")));
        assert!(!CONFIG_EXTENSIONS.matches(Path::new("config.txt")));
    }

    #[test]
    fn test_new() {
        let custom = SupportedExtension::new(&["json", "xml"]);
        assert!(custom.matches(Path::new("data.json")));
        assert!(custom.matches(Path::new("data.xml")));
        assert!(!custom.matches(Path::new("data.txt")));
    }

    #[test]
    fn test_empty_extensions() {
        let empty = SupportedExtension::new(&[]);
        assert!(!empty.matches(Path::new("test.rs")));
        assert!(!empty.matches(Path::new("test.txt")));
    }

    #[test]
    fn test_path_without_extension() {
        assert!(!SUPPORTED.matches(Path::new("noextension")));
        assert!(!SUPPORTED.matches(Path::new("/path/to/file")));
    }

    #[test]
    fn test_debug_impl() {
        let formatted = format!("{:?}", SUPPORTED);
        assert!(formatted.contains("SupportedExtension"));
        assert!(formatted.contains("extensions"));
    }
}

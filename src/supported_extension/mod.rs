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

    const SUPPORTED: SupportedExtension = SupportedExtension::new(&["rs", "toml", "md"]);

    #[test]
    fn test_contains() {
        assert!(SUPPORTED.contains("rs"));
        assert!(SUPPORTED.contains("RS"));
        assert!(SUPPORTED.contains("Md"));
        assert!(!SUPPORTED.contains("exe"));
    }

    #[test]
    fn test_matches() {
        assert!(SUPPORTED.matches(Path::new("foo.rs")));
        assert!(SUPPORTED.matches(Path::new("bar.TOML")));
        assert!(SUPPORTED.matches(Path::new("/baz/qux.md")));
        assert!(!SUPPORTED.matches(Path::new("noext")));
        assert!(!SUPPORTED.matches(Path::new("foo.exe")));
    }
}

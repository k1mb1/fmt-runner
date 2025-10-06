use crate::parser::LanguageProvider;
use crate::supported_extension::SupportedExtension;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// File collector responsible for gathering supported files from the filesystem.
pub struct FileCollector;

impl FileCollector {
    /// Collect unique supported files from multiple paths.
    ///
    /// This function deduplicates files and returns them in sorted order.
    ///
    /// # Arguments
    /// * `paths` - Array of paths to search
    ///
    /// # Returns
    /// Sorted vector of unique file paths
    pub fn collect_all<Language: LanguageProvider>(paths: &[PathBuf]) -> Vec<PathBuf> {
        let mut files_set = HashSet::new();
        let mut files_vec = Vec::new();

        for path in paths {
            for file in Self::collect_from_path::<Language>(path) {
                if files_set.insert(file.clone()) {
                    files_vec.push(file);
                }
            }
        }

        files_vec
    }

    /// Collect supported files from path (file or directory).
    ///
    /// # Arguments
    /// * `root` - Root path to search from
    ///
    /// # Returns
    /// Vector of supported file paths
    fn collect_from_path<Language: LanguageProvider>(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let supported = Language::supported_extension();

        if root.is_file() {
            if supported.matches(root) {
                files.push(root.to_path_buf());
            }
        } else if root.is_dir() {
            Self::collect_recursive(root, supported, &mut files);
        }

        files
    }

    /// Helper: recursively walk directory and push supported files.
    fn collect_recursive(dir: &Path, supported: &SupportedExtension, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_recursive(&path, supported, files);
                } else if supported.matches(&path) {
                    files.push(path);
                }
            }
        }
    }
}

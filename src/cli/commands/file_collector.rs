use crate::parser::LanguageProvider;
use crate::supported_extension::SupportedExtension;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// File collector responsible for gathering supported files from the filesystem.
pub struct FileCollector;

impl FileCollector {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LanguageProvider;
    use rstest::{fixture, rstest};
    use std::fs;
    use tempfile::TempDir;
    use tree_sitter::Language;

    struct MockLanguage;

    impl LanguageProvider for MockLanguage {
        fn language() -> Language {
            unsafe { Language::from_raw(std::ptr::null()) }
        }

        fn supported_extension() -> &'static SupportedExtension {
            static MOCK_EXTENSIONS: SupportedExtension = SupportedExtension::new(&["mock", "test"]);
            &MOCK_EXTENSIONS
        }
    }

    #[fixture]
    fn test_files_structure() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base = temp_dir.path();

        fs::write(base.join("file1.mock"), "content1").unwrap();
        fs::write(base.join("file2.test"), "content2").unwrap();
        fs::write(base.join("file3.txt"), "content3").unwrap();
        fs::write(base.join("file4.rs"), "content4").unwrap();

        let nested = base.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("nested1.mock"), "nested content").unwrap();
        fs::write(nested.join("nested2.test"), "nested content").unwrap();
        fs::write(nested.join("unsupported.xml"), "xml content").unwrap();

        let deep = nested.join("deep");
        fs::create_dir(&deep).unwrap();
        fs::write(deep.join("deep1.mock"), "deep content").unwrap();

        temp_dir
    }

    #[rstest]
    fn test_collect_all_from_single_directory(test_files_structure: TempDir) {
        let paths = vec![test_files_structure.path().to_path_buf()];
        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 5);
        assert!(files.iter().all(|f| f
            .extension()
            .is_some_and(|ext| ext == "mock" || ext == "test")));
    }

    #[rstest]
    fn test_collect_all_from_multiple_paths(test_files_structure: TempDir) {
        let base = test_files_structure.path();
        let paths = vec![base.join("file1.mock"), base.join("nested")];

        let files = FileCollector::collect_all::<MockLanguage>(&paths);
        assert_eq!(files.len(), 4);
    }

    #[rstest]
    fn test_collect_all_deduplicates_files(test_files_structure: TempDir) {
        let base = test_files_structure.path();
        let file_path = base.join("file1.mock");

        let paths = vec![file_path.clone(), file_path.clone(), base.to_path_buf()];

        let files = FileCollector::collect_all::<MockLanguage>(&paths);
        let file1_count = files.iter().filter(|f| f.ends_with("file1.mock")).count();
        assert_eq!(file1_count, 1);
    }

    #[rstest]
    fn test_collect_from_single_file(test_files_structure: TempDir) {
        let file_path = test_files_structure.path().join("file1.mock");
        let paths = vec![file_path.clone()];

        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[rstest]
    fn test_collect_from_unsupported_file() {
        let temp_dir = TempDir::new().unwrap();
        let unsupported = temp_dir.path().join("file.txt");
        fs::write(&unsupported, "content").unwrap();

        let paths = vec![unsupported];
        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 0);
    }

    #[rstest]
    fn test_collect_from_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let paths = vec![temp_dir.path().to_path_buf()];

        let files = FileCollector::collect_all::<MockLanguage>(&paths);
        assert_eq!(files.len(), 0);
    }

    #[rstest]
    fn test_collect_from_nonexistent_path() {
        let paths = vec![PathBuf::from("/nonexistent/path")];
        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 0);
    }

    #[rstest]
    fn test_collect_recursive_depth(test_files_structure: TempDir) {
        let nested_path = test_files_structure.path().join("nested");
        let paths = vec![nested_path];

        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("nested1.mock")));
        assert!(files.iter().any(|f| f.ends_with("nested2.test")));
        assert!(files.iter().any(|f| f.ends_with("deep1.mock")));
    }

    #[rstest]
    fn test_collect_mixed_files_and_directories(test_files_structure: TempDir) {
        let base = test_files_structure.path();
        let paths = vec![
            base.join("file1.mock"),
            base.join("file2.test"),
            base.join("nested"),
            base.join("file3.txt"),
        ];

        let files = FileCollector::collect_all::<MockLanguage>(&paths);
        assert_eq!(files.len(), 5);
    }

    #[rstest]
    fn test_collect_case_sensitivity() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        fs::write(base.join("file1.MOCK"), "content").unwrap();
        fs::write(base.join("file2.Mock"), "content").unwrap();
        fs::write(base.join("file3.TEST"), "content").unwrap();

        let paths = vec![base.to_path_buf()];
        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 3);
    }

    #[rstest]
    fn test_collect_empty_paths_array() {
        let paths: Vec<PathBuf> = vec![];
        let files = FileCollector::collect_all::<MockLanguage>(&paths);

        assert_eq!(files.len(), 0);
    }
}

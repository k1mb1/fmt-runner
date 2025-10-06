use crate::cli::error::CliResult;
use log::debug;
use std::fs::{read_to_string, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

/// File reader with optimizations for large files.
pub struct FileReader {
    /// Buffer size for reading files (default: 8KB)
    buffer_size: usize,
    /// Maximum file size for in-memory reading (default: 10MB)
    max_in_memory_size: usize,
}

impl Default for FileReader {
    fn default() -> Self {
        Self {
            buffer_size: 8 * 1024,                // 8KB buffer
            max_in_memory_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl FileReader {
    /// Read given files into strings with optimization for large files.
    ///
    /// # Arguments
    /// * `files` - Array of file paths to read
    ///
    /// # Returns
    /// Vector of file contents as strings, or first IO error encountered
    pub fn read_files(&self, files: &[PathBuf]) -> CliResult<Vec<String>> {
        let mut contents = Vec::with_capacity(files.len());

        for file_path in files {
            let content = self.read_file(file_path)?;
            contents.push(content);
        }

        Ok(contents)
    }

    /// Read a single file with optimization for large files.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to read
    ///
    /// # Returns
    /// File content as a string
    fn read_file(&self, file_path: &PathBuf) -> CliResult<String> {
        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len() as usize;

        if file_size > self.max_in_memory_size {
            debug!(
                "Reading large file ({} bytes) with buffering: {}",
                file_size,
                file_path.display()
            );
            self.read_large_file(file_path, file_size)
        } else {
            Ok(read_to_string(file_path)?)
        }
    }

    /// Read a large file with buffering.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to read
    /// * `file_size` - Size of the file in bytes
    ///
    /// # Returns
    /// File content as a string
    fn read_large_file(&self, file_path: &PathBuf, file_size: usize) -> CliResult<String> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::with_capacity(self.buffer_size, file);
        let mut content = String::with_capacity(file_size);

        reader.read_to_string(&mut content)?;
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use std::fs;
    use tempfile::TempDir;

    #[fixture]
    fn temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    fn create_sized_file(dir: &TempDir, name: &str, size: usize) -> PathBuf {
        let path = dir.path().join(name);
        let content = "a".repeat(size);
        fs::write(&path, content).unwrap();
        path
    }

    #[rstest]
    fn test_read_single_file(temp_dir: TempDir) {
        let content = "Hello, World!";
        let path = create_test_file(&temp_dir, "test.txt", content);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], content);
    }

    #[rstest]
    fn test_read_multiple_files(temp_dir: TempDir) {
        let content1 = "First file content";
        let content2 = "Second file content";
        let content3 = "Third file content";

        let path1 = create_test_file(&temp_dir, "file1.txt", content1);
        let path2 = create_test_file(&temp_dir, "file2.txt", content2);
        let path3 = create_test_file(&temp_dir, "file3.txt", content3);

        let reader = FileReader::default();
        let files = vec![path1, path2, path3];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], content1);
        assert_eq!(result[1], content2);
        assert_eq!(result[2], content3);
    }

    #[rstest]
    fn test_read_empty_file(temp_dir: TempDir) {
        let path = create_test_file(&temp_dir, "empty.txt", "");

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "");
    }

    #[rstest]
    fn test_read_file_with_unicode(temp_dir: TempDir) {
        let content = "Привет, мир! 🌍 こんにちは世界";
        let path = create_test_file(&temp_dir, "unicode.txt", content);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], content);
    }

    #[rstest]
    fn test_read_file_with_newlines(temp_dir: TempDir) {
        let content = "Line 1\nLine 2\nLine 3\n";
        let path = create_test_file(&temp_dir, "multiline.txt", content);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], content);
    }

    #[rstest]
    fn test_read_small_file_uses_read_to_string(temp_dir: TempDir) {
        let size = 1024; // 1KB
        let path = create_sized_file(&temp_dir, "small.txt", size);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), size);
    }

    #[rstest]
    fn test_read_large_file_uses_buffering(temp_dir: TempDir) {
        let size = 11 * 1024 * 1024; // 11MB
        let path = create_sized_file(&temp_dir, "large.txt", size);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), size);
    }

    #[rstest]
    fn test_read_nonexistent_file() {
        let reader = FileReader::default();
        let files = vec![PathBuf::from("/nonexistent/file.txt")];
        let result = reader.read_files(&files);

        assert!(result.is_err());
    }

    #[rstest]
    fn test_read_empty_files_array() {
        let reader = FileReader::default();
        let files: Vec<PathBuf> = vec![];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn test_read_files_preserves_order(temp_dir: TempDir) {
        let path1 = create_test_file(&temp_dir, "file1.txt", "Content 1");
        let path2 = create_test_file(&temp_dir, "file2.txt", "Content 2");
        let path3 = create_test_file(&temp_dir, "file3.txt", "Content 3");

        let reader = FileReader::default();
        let files = vec![path1, path2, path3];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result[0], "Content 1");
        assert_eq!(result[1], "Content 2");
        assert_eq!(result[2], "Content 3");
    }

    #[rstest]
    fn test_read_file_with_special_characters(temp_dir: TempDir) {
        let content = "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?`~\n\t\r";
        let path = create_test_file(&temp_dir, "special.txt", content);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result[0], content);
    }

    #[rstest]
    fn test_read_files_stops_on_first_error(temp_dir: TempDir) {
        let path1 = create_test_file(&temp_dir, "file1.txt", "Content 1");
        let path2 = PathBuf::from("/nonexistent/file.txt");
        let path3 = create_test_file(&temp_dir, "file3.txt", "Content 3");

        let reader = FileReader::default();
        let files = vec![path1, path2, path3];
        let result = reader.read_files(&files);

        assert!(result.is_err());
    }

    #[rstest]
    #[case(1024)] // 1KB
    #[case(8 * 1024)] // 8KB (buffer size)
    #[case(1024 * 1024)] // 1MB
    fn test_read_various_file_sizes(temp_dir: TempDir, #[case] size: usize) {
        let path = create_sized_file(&temp_dir, "sized.txt", size);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result[0].len(), size);
        assert!(result[0].chars().all(|c| c == 'a'));
    }

    #[rstest]
    fn test_read_file_at_boundary_size(temp_dir: TempDir) {
        let size = 10 * 1024 * 1024; // 10MB
        let path = create_sized_file(&temp_dir, "boundary.txt", size);

        let reader = FileReader::default();
        let files = vec![path];
        let result = reader.read_files(&files).unwrap();

        assert_eq!(result[0].len(), size);
    }
}

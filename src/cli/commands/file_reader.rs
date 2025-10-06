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

use crate::cli::commands::{ConfigLoader, FileCollector, FileReader};
use crate::cli::error::CliResult;
use crate::core::Engine;
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Execute the format command - format files and write changes to disk.
pub fn execute<Language, Config>(
    config_path: &Path,
    files_path: &[PathBuf],
    pipeline: Pipeline<Config>,
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config = ConfigLoader::load::<Config>(config_path)?;

    let files = FileCollector::collect_all::<Language>(files_path);

    if files.is_empty() {
        info!("No supported files found to format.");
        return Ok(());
    }

    info!("Found {} file(s) to format", files.len());

    let reader = FileReader::default();
    let file_contents = reader.read_files(&files)?;

    let mut engine = Engine::<Language, Config>::new(pipeline);

    info!("Running in format mode...");
    let changed_files = engine.format_and_write(&config, &file_contents, &files)?;

    if changed_files.is_empty() {
        info!("✓ No files needed formatting!");
    } else {
        info!("✓ Successfully formatted {} file(s):", changed_files.len());
        for file in &changed_files {
            info!("  - {}", file.display());
        }
    }

    Ok(())
}

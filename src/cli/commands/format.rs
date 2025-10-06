use crate::cli::commands::{ConfigLoader, FileCollector, FileReader};
use crate::cli::error::CliResult;
use crate::core::Engine;
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use log::{info, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Execute the format or check command with improved architecture and performance.
pub fn execute<Language, Config>(
    config_path: &Path,
    files_path: &[PathBuf],
    pipeline: Pipeline<Config>,
    write_changes: bool,
    show_diff: bool,
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

    info!("Found {} file(s) to process", files.len());

    let reader = FileReader::default();
    let file_contents = reader.read_files(&files)?;

    let mut engine = Engine::<Language, Config>::new(pipeline);

    if write_changes {
        execute_write_mode(&mut engine, &config, &file_contents, &files)?;
    } else {
        execute_check_mode(&mut engine, &config, &file_contents, &files, show_diff);
    }

    Ok(())
}

/// Execute check mode - verify if files need formatting.
fn execute_check_mode<Language, Config>(
    engine: &mut Engine<Language, Config>,
    config: &Config,
    file_contents: &[String],
    files: &[PathBuf],
    show_diff: bool,
) where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    info!("Running in check mode...");
    let changed_files = engine.check(config, file_contents, files);

    if changed_files.is_empty() {
        info!("✓ All files are formatted correctly!");
    } else {
        warn!(
            "✗ The following {} file(s) need formatting:",
            changed_files.len()
        );
        for file in &changed_files {
            warn!("  - {}", file.display());
        }
        
        if show_diff {
            warn!("\nDifferences:");
            // TODO: Implement diff display
            warn!("  (diff display not yet implemented)");
        }
        
        info!("\nRun 'format' command to apply formatting.");
    }
}

/// Execute write mode - format and write files.
fn execute_write_mode<Language, Config>(
    engine: &mut Engine<Language, Config>,
    config: &Config,
    file_contents: &[String],
    files: &[PathBuf],
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    info!("Running in write mode...");
    let changed_files = engine.format_and_write(config, file_contents, files)?;

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

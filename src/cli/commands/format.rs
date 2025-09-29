use crate::cli::cli_entry::FormatMode;
use crate::cli::commands::utils::{
    collect_all_supported_files, load_config, read_files_to_strings,
};
use crate::cli::error::CliResult;
use crate::core::Engine;
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use log::{info, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub fn execute<Language, Config>(
    config_path: &Path,
    files_path: &[PathBuf],
    pipeline: Pipeline<Config>,
    mode: FormatMode,
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config = load_config::<Config>(config_path)?;

    let files = collect_all_supported_files::<Language>(files_path);
    let file_contents = read_files_to_strings(&files)?;

    let mut engine = Engine::<Language, Config>::new(pipeline);

    match mode {
        FormatMode::Check => {
            info!("Running in check mode...");
            let changed_files = engine.check(&config, &file_contents, &files);

            if changed_files.is_empty() {
                info!("✓ All files are formatted correctly!");
            } else {
                warn!("✗ The following files need formatting:");
                for file in &changed_files {
                    warn!("  - {}", file.display());
                }
                info!("\nRun with --mode write to apply formatting.");
            }
        }
        FormatMode::Write => {
            info!("Running in write mode...");
            let changed_files = engine.format_and_write(&config, &file_contents, &files)?;

            if changed_files.is_empty() {
                info!("✓ No files needed formatting!");
            } else {
                info!("✓ Formatted {} file(s):", changed_files.len());
                for file in &changed_files {
                    info!("  - {}", file.display());
                }
            }
        }
    }

    Ok(())
}

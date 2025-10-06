use crate::cli::commands::{ConfigLoader, FileCollector, FileReader};
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
    show_diff: bool,
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config = ConfigLoader::load::<Config>(config_path)?;

    let files = FileCollector::collect_all::<Language>(files_path);

    if files.is_empty() {
        info!("No supported files found to check.");
        return Ok(());
    }

    info!("Found {} file(s) to check", files.len());

    let reader = FileReader::default();
    let file_contents = reader.read_files(&files)?;

    let mut engine = Engine::<Language, Config>::new(pipeline);

    info!("Running in check mode...");
    let changed_files = engine.check(&config, &file_contents, &files);

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

    Ok(())
}

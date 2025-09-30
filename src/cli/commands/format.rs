use crate::cli::cli_entry::FormatMode;
use crate::cli::commands::utils::{
    collect_all_supported_files, load_config, read_files_to_strings,
};
use crate::cli::error::CliResult;
use crate::core::{DiagnosticSeverity, Engine, FileFormatOutcome};
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
            let outcomes = engine.check(&config, &file_contents, &files);
            log_diagnostics(&outcomes);

            let changed: Vec<_> = outcomes.iter().filter(|o| o.changed).collect();

            if changed.is_empty() {
                info!("✓ All files are formatted correctly!");
            } else {
                warn!("✗ The following files need formatting:");
                for outcome in changed {
                    let display = outcome
                        .path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string());
                    warn!("  - {display}");
                }
                info!("\nRun with --mode write to apply formatting.");
            }
        }
        FormatMode::Diff => {
            info!("Running in diff mode...");
            let outcomes = engine.check(&config, &file_contents, &files);
            log_diagnostics(&outcomes);

            let mut changed_count = 0usize;

            for outcome in &outcomes {
                if !outcome.changed {
                    continue;
                }
                changed_count += 1;

                let display = outcome
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "<unknown>".to_string());

                println!("--- diff: {display}");
                if let Some(diff) = &outcome.diff {
                    println!("{diff}");
                } else {
                    println!("(diff unavailable)");
                }
                println!();
            }

            if changed_count == 0 {
                info!("✓ No differences found.");
            } else {
                warn!("✗ {} file(s) would change.", changed_count);
                info!("Run with --mode write to apply these edits.");
            }
        }
        FormatMode::Write => {
            info!("Running in write mode...");
            let outcomes = engine.format_and_write(&config, &file_contents, &files)?;
            log_diagnostics(&outcomes);

            let changed: Vec<_> = outcomes.iter().filter(|o| o.changed).collect();

            if changed.is_empty() {
                info!("✓ No files needed formatting!");
            } else {
                info!("✓ Formatted {} file(s):", changed.len());
                for outcome in changed {
                    let display = outcome
                        .path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string());
                    info!("  - {display}");
                }
            }
        }
    }

    Ok(())
}

fn log_diagnostics(outcomes: &[FileFormatOutcome]) {
    for outcome in outcomes {
        let path_display = outcome
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        for diagnostic in &outcome.diagnostics {
            let source = diagnostic
                .source
                .as_deref()
                .unwrap_or("engine");

            match diagnostic.severity {
                DiagnosticSeverity::Info => {
                    info!(
                        "[{}] {}: {}",
                        source,
                        path_display,
                        diagnostic.message
                    );
                }
                DiagnosticSeverity::Warning => {
                    warn!(
                        "[{}] {}: {}",
                        source,
                        path_display,
                        diagnostic.message
                    );
                }
                DiagnosticSeverity::Error => {
                    log::error!(
                        "[{}] {}: {}",
                        source,
                        path_display,
                        diagnostic.message
                    );
                }
            }
        }
    }
}

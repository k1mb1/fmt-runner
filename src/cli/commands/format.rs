use crate::cli::commands::utils::{
    collect_all_supported_files, load_config, read_files_to_strings,
};
use crate::cli::error::CliResult;
use crate::cli::cli_entry::FormatMode;
use crate::core::Engine;
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::PathBuf;

pub fn execute<Language, Config>(
    config_path: PathBuf,
    files_path: Vec<PathBuf>,
    pipeline: Pipeline<Config>,
    mode: FormatMode,
) -> CliResult<()>
where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config = load_config::<Config>(config_path.as_path())?;

    let files = collect_all_supported_files::<Language>(&files_path);
    let file_contents = read_files_to_strings(&files)?;

    let mut engine = Engine::<Language, Config>::new(pipeline);
    
    match mode {
        FormatMode::Check => {
            println!("Running in check mode...");
            let changed_files = engine.check(&config, &file_contents, &files);
            
            if changed_files.is_empty() {
                println!("✓ All files are formatted correctly!");
            } else {
                println!("✗ The following files need formatting:");
                for file in &changed_files {
                    println!("  - {}", file.display());
                }
                println!("\nRun with --mode write to apply formatting.");
            }
        }
        FormatMode::Write => {
            println!("Running in write mode...");
            let changed_files = engine.format_and_write(&config, &file_contents, &files)?;
            
            if changed_files.is_empty() {
                println!("✓ No files needed formatting!");
            } else {
                println!("✓ Formatted {} file(s):", changed_files.len());
                for file in &changed_files {
                    println!("  - {}", file.display());
                }
            }
        }
    }

    Ok(())
}

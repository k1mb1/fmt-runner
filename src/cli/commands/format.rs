use crate::cli::commands::utils::{
    collect_all_supported_files, load_config, read_files_to_strings,
};
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
) where
    Config: Serialize + DeserializeOwned + Default,
    Language: LanguageProvider,
{
    let config = match load_config::<Config>(config_path.as_path()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to initialize config: {}", e);
            return;
        }
    };

    //TODO if files_path is not dirs_path, then we need to handle that case
    let files = collect_all_supported_files::<Language>(&files_path);
    let file_contents = read_files_to_strings(&files);

    let mut engine = Engine::<Language, Config>::new(pipeline);
    println!("start");
    engine.start(&config, &file_contents)

}

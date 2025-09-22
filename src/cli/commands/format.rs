use crate::cli::commands::utils::{
    collect_all_supported_files, load_config, read_files_to_strings,
};
use crate::core::Engine;
use crate::parser::LanguageProvider;
use crate::pipeline::Pipeline;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::PathBuf;


pub fn run<L, C>(config_path: PathBuf, files_path: Vec<PathBuf>, pipeline: Pipeline<C>)
where
    C: Serialize + DeserializeOwned + Default,
    L: LanguageProvider,
{
    let config = match load_config::<C>(config_path.as_path()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to initialize config: {}", e);
            return;
        }
    };

    let files = collect_all_supported_files::<L>(&files_path);
    let file_contents = read_files_to_strings(&files);

    let mut engine = Engine::<L, C>::new(pipeline);
    engine.start(&config, &file_contents)
}

use crate::cli::commands::utils::init_config;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;


pub fn run<Config>(config_path: PathBuf)
where
    Config: Serialize + DeserializeOwned + Default,
{
    if let Err(e) = init_config::<Config>(config_path.as_path()) {
        eprintln!("Failed to initialize config: {}", e);
    }
}

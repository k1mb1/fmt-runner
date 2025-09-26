use crate::cli::commands::utils::init_config;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub fn execute<Config>(config_path: PathBuf)
where
    Config: Serialize + DeserializeOwned + Default,
{
    //TODO validate config

    //TODO if validate what is value missing

    if let Err(e) = init_config::<Config>(config_path.as_path()) {
        eprintln!("Failed to initialize config: {}", e);
    }
}

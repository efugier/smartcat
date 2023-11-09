use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    #[serde(skip_serializing)] // internal use only
    pub api_key: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Prompt {
    #[serde(skip_serializing)] // internal use only
    pub api: String,
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub const PLACEHOLDER_TOKEN: &str = "#[<input>]";

const DEFAULT_CONFIG_PATH: &str = ".config/pipelm/";
const CUSTOM_CONFIG_ENV_VAR: &str = "PIPELM_CONFIG_PATH";
const API_KEYS_FILE: &str = ".api_configs.toml";
const PROMPT_FILE: &str = "prompts.toml";

fn resolve_config_path() -> PathBuf {
    match std::env::var(CUSTOM_CONFIG_ENV_VAR) {
        Ok(p) => PathBuf::new().join(p),
        Err(_) => PathBuf::new().join(env!("HOME")).join(DEFAULT_CONFIG_PATH),
    }
}

pub fn get_api_config(api: &str) -> ApiConfig {
    let api_keys_path = resolve_config_path().join(API_KEYS_FILE);
    let content = fs::read_to_string(&api_keys_path)
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", api_keys_path, error));

    let mut api_configs: HashMap<String, ApiConfig> = toml::from_str(&content).unwrap();

    api_configs.remove(api).unwrap_or_else(|| {
        panic!(
            "Prompt {} not found, availables ones are: {:?}",
            api,
            api_configs.keys().collect::<Vec<_>>()
        )
    })
}

pub fn get_prompts() -> HashMap<String, Prompt> {
    let prompts_path = resolve_config_path().join(PROMPT_FILE);
    let content = fs::read_to_string(&prompts_path)
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", prompts_path, error));
    toml::from_str(&content).unwrap()
}

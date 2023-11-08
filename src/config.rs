use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Prompt {
    #[serde(skip_serializing)] // internal use only
    pub service: String,
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
const API_KEYS_FILE: &str = ".api_keys.toml";
const PROMPT_FILE: &str = "prompts.toml";

fn resolve_config_path() -> PathBuf {
    match std::env::var(CUSTOM_CONFIG_ENV_VAR) {
        Ok(p) => PathBuf::new().join(p),
        Err(_) => PathBuf::new().join(env!("HOME")).join(DEFAULT_CONFIG_PATH),
    }
}

pub fn get_api_key(service: &str) -> String {
    let api_keys_path = resolve_config_path().join(API_KEYS_FILE);
    let content = fs::read_to_string(&api_keys_path)
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", api_keys_path, error));
    let value: Value = content.parse().expect("Failed to parse TOML");

    // Extract the API key from the TOML table.
    let api_key = value
        .get("API_KEYS")
        .expect("API_KEYS section not found")
        .get(service)
        .unwrap_or_else(|| panic!("No api key found for service {}.", &service));

    api_key.to_string()
}

pub fn get_prompts() -> HashMap<String, Prompt> {
    let prompts_path = resolve_config_path().join(PROMPT_FILE);
    let content = fs::read_to_string(&prompts_path)
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", prompts_path, error));
    toml::from_str(&content).unwrap()
}

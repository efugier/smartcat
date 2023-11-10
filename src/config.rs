use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

pub const PLACEHOLDER_TOKEN: &str = "#[<input>]";

const DEFAULT_CONFIG_PATH: &str = ".config/smartcat/";
const CUSTOM_CONFIG_ENV_VAR: &str = "PIPELM_CONFIG_PATH";
const API_KEYS_FILE: &str = ".api_configs.toml";
const PROMPT_FILE: &str = "prompts.toml";

#[derive(clap::ValueEnum, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Api {
    Openai,
}

impl FromStr for Api {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Api::Openai),
            _ => Err(()),
        }
    }
}

impl ToString for Api {
    fn to_string(&self) -> String {
        match self {
            Api::Openai => "openai".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiConfig {
    pub api_key: String,
    pub url: String,
}

impl Default for ApiConfig {
    // default to openai
    fn default() -> Self {
        ApiConfig {
            api_key: String::from("<insert_api_key_here>"),
            url: String::from("https://api.openai.com/v1/chat/completions"),
        }
    }
}

impl ApiConfig {
    fn default_with_api_key(api_key: String) -> Self {
        ApiConfig {
            api_key,
            url: String::from("https://api.openai.com/v1/chat/completions"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Prompt {
    pub api: Api,
    pub model: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<Message>,
}

impl Default for Prompt {
    /// default to openai and gpt 4 with a preset message telling the
    /// model to behave like smart version of cat.
    fn default() -> Self {
        let messages = vec![Message {
                role: "system".to_string(),
                content: "\
                    You are an extremely skilled programmer with a keen eye for detail and an emphasis on readable code. \
                    You have been tasked with acting as a smart version of the cat unix program. You take text and a prompt in and write text out. \
                    For that reason, it is of crucial importance to just write the desired output. Do not under any circumstance write any comment or thought \
                    as you output will be piped into other programs. Do not write the markdown delimiters for code as well. \
                    Sometimes you will be asked to implement or extend some input code. Same thing goes here, write only what was asked because what you write will \
                    be directly added to the user's editor. \
                    Never ever write ``` around the code. \
                    Now let's make something great together!
                ".to_string(),
            }
        ];
        Prompt {
            api: Api::Openai,
            model: String::from("gpt-4"),
            messages,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

fn resolve_config_path() -> PathBuf {
    match std::env::var(CUSTOM_CONFIG_ENV_VAR) {
        Ok(p) => PathBuf::new().join(p),
        Err(_) => PathBuf::new().join(env!("HOME")).join(DEFAULT_CONFIG_PATH),
    }
}
fn prompts_path() -> PathBuf {
    resolve_config_path().join(PROMPT_FILE)
}
fn api_keys_path() -> PathBuf {
    resolve_config_path().join(API_KEYS_FILE)
}

pub fn get_api_config(api: &str) -> ApiConfig {
    let content = fs::read_to_string(api_keys_path())
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", api_keys_path(), error));

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
    let content = fs::read_to_string(prompts_path())
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", prompts_path(), error));
    toml::from_str(&content).unwrap()
}

fn read_user_input() -> String {
    let mut user_input = String::new();
    std::io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");
    user_input.trim().to_string()
}

fn prompt_user_for_config_file_creation(file_path: impl Debug) {
    println!(
        "Api config file not found at {:?}, do you wish to generate one? [y/n]",
        file_path
    );
    if read_user_input().to_lowercase() != "y" {
        println!("smartcat needs this file tu function, create it and come back 👋");
        std::process::exit(1);
    }
}

pub fn ensure_config_files() -> std::io::Result<()> {
    if !api_keys_path().exists() {
        prompt_user_for_config_file_creation(api_keys_path());
        println!(
            "Please paste your openai API key, it can be found at\n\
                https://platform.openai.com/api-keys\n\
                Press enter to skip (then edit the file at {:?}).",
            api_keys_path()
        );
        let mut api_config = HashMap::new();
        api_config.insert(
            Prompt::default().api.to_string(),
            ApiConfig::default_with_api_key(read_user_input()),
        );

        let api_config_str = toml::to_string_pretty(&api_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut config_file = fs::File::create(api_keys_path())?;
        config_file.write_all(api_config_str.as_bytes())?;
    }

    if !prompts_path().exists() {
        prompt_user_for_config_file_creation(prompts_path());
        let mut prompt_config = HashMap::new();
        prompt_config.insert("default", Prompt::default());
        let prompt_str = toml::to_string_pretty(&prompt_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut prompts_file = fs::File::create(prompts_path())?;
        prompts_file.write_all(prompt_str.as_bytes())?;
    }

    Ok(())
}

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
const CUSTOM_CONFIG_ENV_VAR: &str = "SMARTCAT_CONFIG_PATH";
const API_KEYS_FILE: &str = ".api_configs.toml";
const PROMPT_FILE: &str = "prompts.toml";
const CONVERSATION_FILE: &str = "conversation.toml";

#[derive(clap::ValueEnum, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Api {
    Openai,
    Mistral,
    AnotherApiForTests,
}

impl FromStr for Api {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Api::Openai),
            "mistral" => Ok(Api::Mistral),
            _ => Err(()),
        }
    }
}

impl ToString for Api {
    fn to_string(&self) -> String {
        match self {
            Api::Openai => "openai".to_string(),
            Api::Mistral => "mistral".to_string(),
            v => panic!(
                "{:?} is not implemented, use one among {:?}",
                v,
                vec![Api::Openai]
            ),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ApiConfig {
    pub api_key_command: Option<String>,
    pub api_key: Option<String>,
    pub url: String,
    pub default_model: Option<String>,
}

impl Default for ApiConfig {
    // default to openai
    fn default() -> Self {
        ApiConfig::openai()
    }
}

impl ApiConfig {
    pub fn get_api_key(&self) -> String {
        self.api_key
            .clone()
            .or_else(|| {
                self.api_key_command.clone().map(|command| {
                    let output =
                        std::process::Command::new(command.split_whitespace().next().unwrap())
                            .args(command.split_whitespace().skip(1))
                            .output()
                            .expect("Failed to run the api command")
                            .stdout;
                    String::from_utf8(output)
                        .expect("Invalid UTF-8 from command")
                        .trim()
                        .to_string()
                })
            })
            .expect("No api_key found.")
    }

    fn openai() -> Self {
        ApiConfig {
            api_key_command: None,
            api_key: None,
            url: String::from("https://api.openai.com/v1/chat/completions"),
            default_model: Some(String::from("gpt-4")),
        }
    }

    fn mistral() -> Self {
        ApiConfig {
            api_key_command: None,
            api_key: None,
            url: String::from("https://api.mistral.ai/v1/chat/completions"),
            default_model: Some(String::from("mistral-medium")),
        }
    }

    fn default_with_api_key(api_key: Option<String>) -> Self {
        ApiConfig {
            api_key_command: None,
            api_key,
            url: String::from("https://api.openai.com/v1/chat/completions"),
            default_model: Some(String::from("gpt-4")),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Prompt {
    pub api: Api,
    pub model: Option<String>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl Default for Prompt {
    /// default to openai and gpt 4 with a preset message telling the
    /// model to behave like smart version of cat.
    fn default() -> Self {
        let messages = vec![
            Message::system( "\
                You are an extremely skilled programmer with a keen eye for detail and an emphasis on readable code. \
                You have been tasked with acting as a smart version of the cat unix program. You take text and a prompt in and write text out. \
                For that reason, it is of crucial importance to just write the desired output. Do not under any circumstance write any comment or thought \
                as your output will be piped into other programs. Do not write the markdown delimiters for code as well. \
                Sometimes you will be asked to implement or extend some input code. Same thing goes here, write only what was asked because what you write will \
                be directly added to the user's editor. \
                Never ever write ``` around the code. \
                Now let's make something great together! \
            ")
        ];
        Prompt {
            api: Api::Openai,
            model: None,
            temperature: None,
            messages,
        }
    }
}

impl Prompt {
    pub fn empty() -> Self {
        let default_prompt = Prompt::default();
        Prompt {
            api: default_prompt.api,
            model: default_prompt.model,
            temperature: None,
            messages: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn user(content: &str) -> Message {
        Message {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }
    pub fn system(content: &str) -> Message {
        Message {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }
    pub fn assistant(content: &str) -> Message {
        Message {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
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
pub fn conversation_file_path() -> PathBuf {
    resolve_config_path().join(CONVERSATION_FILE)
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
    toml::from_str(&content).expect("could not parse prompt file content")
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
        "Config file not found at {:?}, do you wish to generate one? [y/n]",
        file_path
    );
    if read_user_input().to_lowercase() != "y" {
        println!("smartcat needs this file tu function, create it and come back 👋");
        std::process::exit(1);
    }
}

pub fn ensure_config_files(interactive: bool) -> std::io::Result<()> {
    let mut config_was_generated = false;
    if !api_keys_path().exists() {
        let openai_api_key = if interactive {
            prompt_user_for_config_file_creation(api_keys_path());
            println!(
                "Please paste your openai API key, it can be found at\n\
                https://platform.openai.com/api-keys\n\
                Press [ENTER] to skip"
            );
            let input = read_user_input().trim().to_string();
            if input.trim().is_empty() {
                println!(
                    "Please edit the file at {:?} more \
                    config options are available this way. See\
                    https://github.com/efugier/smartcat#configuration",
                    api_keys_path()
                );
                None
            } else {
                Some(input)
            }
        } else {
            None
        };
        let mut api_config = HashMap::new();
        api_config.insert(
            Prompt::default().api.to_string(),
            ApiConfig::default_with_api_key(openai_api_key),
        );
        api_config.insert(Api::Mistral.to_string(), ApiConfig::mistral());

        let api_config_str = toml::to_string_pretty(&api_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        std::fs::create_dir_all(api_keys_path().parent().unwrap())?;

        let mut config_file = fs::File::create(api_keys_path())?;
        let doc = "\
        # Api config files, use `api_key` or `api_key_command` fields\n\
        # to set the api key for each api\n\
        # more details at https://github.com/efugier/smartcat#configuration\n\n";
        config_file.write_all(doc.as_bytes())?;
        config_file.write_all(api_config_str.as_bytes())?;
        config_was_generated = true;
    }

    if !prompts_path().exists() {
        if interactive {
            prompt_user_for_config_file_creation(prompts_path());
        }
        let mut prompt_config = HashMap::new();
        prompt_config.insert("default", Prompt::default());
        prompt_config.insert("empty", Prompt::empty());

        let prompt_str = toml::to_string_pretty(&prompt_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        std::fs::create_dir_all(prompts_path().parent().unwrap())?;

        let mut prompts_file = fs::File::create(prompts_path())?;
        prompts_file.write_all(prompt_str.as_bytes())?;
        config_was_generated = true;
    }

    if interactive & config_was_generated {
        std::process::exit(0);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    #[test]
    #[serial]
    fn resolver_custom_config_path() {
        let temp_path = "/tmp/custom_path";
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);

        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_path);
        let result = resolve_config_path();

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        assert_eq!(result, Path::new(temp_path));
    }

    #[test]
    #[serial]
    fn resolve_default_config_path() {
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);

        env::remove_var(CUSTOM_CONFIG_ENV_VAR);
        let home_dir = env::var("HOME").expect("HOME not defined");
        let default_path = PathBuf::new().join(home_dir).join(DEFAULT_CONFIG_PATH);
        let result = resolve_config_path();

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        assert_eq!(result, Path::new(&default_path));
    }

    #[test]
    #[serial]
    fn test_ensure_config_files_not_existing() -> std::io::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);
        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_dir.path());

        let api_keys_path = api_keys_path();
        let prompts_path = prompts_path();

        assert!(!api_keys_path.exists());
        assert!(!prompts_path.exists());

        let result = ensure_config_files(false);

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        result?;

        assert!(api_keys_path.exists());
        assert!(prompts_path.exists());
        Ok(())
    }

    #[test]
    #[serial]
    fn test_ensure_config_files_already_existing() -> std::io::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;

        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);
        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_dir.path());

        let api_keys_path = api_keys_path();
        let prompts_path = prompts_path();

        // Precreate files with some content
        let mut api_keys_file = fs::File::create(&api_keys_path)?;
        api_keys_file.write_all(b"Some API key data")?;

        let mut prompts_file = fs::File::create(&prompts_path)?;
        prompts_file.write_all(b"Some prompts data")?;

        let result = ensure_config_files(false);

        // Restoring the original environment variable
        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        result?;

        // Check if files still exist
        assert!(api_keys_path.exists());
        assert!(prompts_path.exists());

        // Check if the contents remain unchanged
        let mut api_keys_content = String::new();
        fs::File::open(&api_keys_path)?.read_to_string(&mut api_keys_content)?;
        assert_eq!(api_keys_content, "Some API key data".to_string());

        let mut prompts_content = String::new();
        fs::File::open(&prompts_path)?.read_to_string(&mut prompts_content)?;
        assert_eq!(prompts_content, "Some prompts data".to_string());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_ensure_config_files_serialization() -> std::io::Result<()> {
        // Setup paths
        let temp_dir = tempfile::TempDir::new()?;
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);
        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_dir.path());

        let api_keys_path = api_keys_path();
        let prompts_path = prompts_path();

        assert!(!api_keys_path.exists());
        assert!(!prompts_path.exists());

        let result = ensure_config_files(false);

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        result?;

        // Read back the files and deserialize
        let api_config_contents = fs::read_to_string(&api_keys_path)?;
        let prompts_config_contents = fs::read_to_string(&prompts_path)?;

        // Deserialize contents to expected data structures
        // TODO: would be better to use `get_config` and `get_prompts` but
        // current implementation does not allow for error management that would
        // enable safe environement variable manipulation
        let api_config: HashMap<String, ApiConfig> =
            toml::from_str(&api_config_contents).expect("Failed to deserialize API config");

        let prompt_config: HashMap<String, Prompt> =
            toml::from_str(&prompts_config_contents).expect("Failed to deserialize prompts config");

        // Check if the content matches the default values
        assert_eq!(
            api_config.get(&Prompt::default().api.to_string()),
            Some(&ApiConfig::default())
        );
        assert_eq!(
            api_config.get(&Api::Mistral.to_string()),
            Some(&ApiConfig::mistral())
        );

        let default_prompt = Prompt::default();
        assert_eq!(prompt_config.get("default"), Some(&default_prompt));

        let empty_prompt = Prompt::empty();
        assert_eq!(prompt_config.get("empty"), Some(&empty_prompt));

        Ok(())
    }
}

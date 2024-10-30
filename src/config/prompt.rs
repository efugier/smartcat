use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::config::{api::Api, resolve_config_path};

const PROMPT_FILE: &str = "prompts.toml";
const CONVERSATION_FILE: &str = "conversation.toml";

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Prompt {
    pub api: Api,
    pub model: Option<String>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>, // unsuported for now
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
                Make sure to keep the indentation and formatting. \
            ")
        ];
        Prompt {
            api: Api::Ollama,
            model: None,
            temperature: None,
            messages,
            stream: None,
            char_limit: Some(50000),
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
            stream: None,
            char_limit: Some(50000),
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

pub(super) fn prompts_path() -> PathBuf {
    resolve_config_path().join(PROMPT_FILE)
}

pub fn conversation_file_path() -> PathBuf {
    resolve_config_path().join(CONVERSATION_FILE)
}

pub fn get_last_conversation_as_prompt() -> Prompt {
    let content = fs::read_to_string(conversation_file_path()).unwrap_or_else(|error| {
        panic!(
            "Could not read file {:?}, {:?}",
            conversation_file_path(),
            error
        )
    });
    toml::from_str(&content).expect("failed to load the conversation file")
}

pub(super) fn generate_prompts_file() -> std::io::Result<()> {
    let mut prompt_config = HashMap::new();
    prompt_config.insert("default", Prompt::default());
    prompt_config.insert("empty", Prompt::empty());

    std::fs::create_dir_all(prompts_path().parent().unwrap())?;

    let mut prompts_file = fs::File::create(prompts_path())?;

    let doc = "\
        # Prompt config files\n\
        # more details and examples at https://github.com/efugier/smartcat#configuration\n\n";
    prompts_file.write_all(doc.as_bytes())?;

    let prompt_str = toml::to_string_pretty(&prompt_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    prompts_file.write_all(prompt_str.as_bytes())?;
    Ok(())
}

pub fn get_prompts() -> HashMap<String, Prompt> {
    let content = fs::read_to_string(prompts_path())
        .unwrap_or_else(|error| panic!("Could not read file {:?}, {:?}", prompts_path(), error));
    toml::from_str(&content).expect("could not parse prompt file content")
}

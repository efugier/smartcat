use crate::config::{Api, Message, Prompt};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::config::ApiConfig;

#[derive(Debug, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    pub index: u32,
    pub message: OpenAiMessage,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
    pub system_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiPrompt {
    pub model: String,
    pub messages: Vec<Message>,
}

impl From<Prompt> for OpenAiPrompt {
    fn from(prompt: Prompt) -> OpenAiPrompt {
        OpenAiPrompt {
            model: prompt.model,
            messages: prompt.messages,
        }
    }
}

pub fn make_authenticated_request(
    api_config: ApiConfig,
    prompt: Prompt,
) -> Result<ureq::Response, ureq::Error> {
    debug!("Trying to reach openai with {}", api_config.api_key);
    debug!("request content: {:?}", prompt);

    let request = ureq::post(&api_config.url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", api_config.api_key));
    match prompt.api {
        Api::Openai => request.send_json(OpenAiPrompt::from(prompt)),
        v => panic!(
            "{:?} is not implemented, use on among {:?}",
            v,
            vec![Api::Openai]
        ),
    }
}

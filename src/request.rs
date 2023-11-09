use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::config::ApiConfig;

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
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
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub system_fingerprint: Option<String>,
}

pub fn make_authenticated_request(
    api_config: ApiConfig,
    data: impl Serialize + Debug,
) -> Result<ureq::Response, ureq::Error> {
    debug!("Trying to reach openai with {}", api_config.api_key);
    debug!("request content: {:?}", data);
    ureq::post(&api_config.url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", api_config.api_key))
        .send_json(data)
}

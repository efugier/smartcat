use crate::config::prompt::Message;
use serde::Deserialize;
use std::fmt::Debug;

#[derive(Debug, Deserialize)]
pub(super) struct AnthropicMessage {
    pub text: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub _type: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct MessageWrapper {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub(super) struct OpenAiResponse {
    pub choices: Vec<MessageWrapper>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AnthropicResponse {
    pub content: Vec<AnthropicMessage>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaResponse {
    pub message: Message,
}

impl From<OllamaResponse> for String {
    fn from(value: OllamaResponse) -> Self {
        value.message.content
    }
}

impl From<AnthropicResponse> for String {
    fn from(value: AnthropicResponse) -> Self {
        value.content.first().unwrap().text.to_owned()
    }
}

impl From<OpenAiResponse> for String {
    fn from(value: OpenAiResponse) -> Self {
        value.choices.first().unwrap().message.content.to_owned()
    }
}

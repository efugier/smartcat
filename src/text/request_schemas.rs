use crate::config::prompt::{Message, Prompt};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct OpenAiPrompt {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct AnthropicPrompt {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl From<Prompt> for OpenAiPrompt {
    fn from(prompt: Prompt) -> OpenAiPrompt {
        OpenAiPrompt {
            model: prompt
                .model
                .expect("model must be specified either in the api config or in the prompt config"),
            messages: prompt.messages,
            temperature: prompt.temperature,
            stream: prompt.stream,
        }
    }
}

impl From<Prompt> for AnthropicPrompt {
    fn from(prompt: Prompt) -> Self {
        let merged_messages =
            prompt
                .messages
                .into_iter()
                .fold(Vec::new(), |mut acc: Vec<Message>, mut message| {
                    if message.role == "system" {
                        message.role = "user".to_string();
                    }
                    match acc.last_mut() {
                        Some(last_message) if last_message.role == message.role => {
                            last_message.content.push_str("\n\n");
                            last_message.content.push_str(&message.content);
                        }
                        _ => acc.push(message),
                    }
                    acc
                });

        AnthropicPrompt {
            model: prompt.model.expect("model must be specified"),
            messages: merged_messages,
            temperature: prompt.temperature,
            stream: prompt.stream,
            max_tokens: 4096,
        }
    }
}

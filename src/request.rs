use crate::config::{Api, ApiConfig, Message, Prompt};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io;

#[derive(Debug, Deserialize)]
pub struct AnthropicMessage {
    pub text: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub _type: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageWrapper {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<MessageWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicMessage>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiPrompt {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnthropicPrompt {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub max_tokens: i32,
}

impl From<Prompt> for OpenAiPrompt {
    fn from(prompt: Prompt) -> OpenAiPrompt {
        OpenAiPrompt {
            model: prompt
                .model
                .expect("model must be specified either in the api config or in the prompt config"),
            messages: prompt.messages,
            temperature: prompt.temperature,
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
            max_tokens: 4096,
        }
    }
}

fn parse_response(response: Result<ureq::Response, ureq::Error>) -> io::Result<ureq::Response> {
    response.map_err(|e| match e {
        ureq::Error::Status(status, response) => {
            let body = match response.into_string() {
                Ok(body) => body,
                Err(_) => "(non-UTF-8 response)".to_owned(),
            };
            io::Error::other(format!(
                "API call failed with status code {status} and body: {body}"
            ))
        }
        ureq::Error::Transport(transport) => io::Error::other(transport),
    })
}

pub fn make_api_request(api_config: ApiConfig, prompt: &Prompt) -> io::Result<Message> {
    debug!(
        "Trying to reach {:?} with {:?}",
        api_config.url, api_config.api_key
    );
    debug!("request content: {:?}", prompt);
    let mut prompt = prompt.clone();

    if prompt.model.is_none() {
        prompt.model = api_config.default_model.clone()
    }

    let request = ureq::post(&api_config.url);
    let response_text = match prompt.api {
        Api::Openai | Api::Mistral => {
            let request = request.set("Content-Type", "application/json").set(
                "Authorization",
                &format!("Bearer {}", &api_config.get_api_key()),
            );
            let response: OpenAiResponse =
                parse_response(request.send_json(OpenAiPrompt::from(prompt)))?.into_json()?;
            response.into()
        }
        Api::Anthropic => {
            let request = request
                .set("Content-Type", "application/json")
                .set("x-api-key", &api_config.get_api_key())
                .set("anthropic-version", "2023-06-01");
            let response: AnthropicResponse =
                parse_response(request.send_json(AnthropicPrompt::from(prompt)))?.into_json()?;
            response.into()
        }
        unknown_api => panic!(
            "{:?} is not implemented, use one among {:?}",
            unknown_api,
            vec![Api::Openai, Api::Mistral, Api::Anthropic]
        ),
    };

    Ok(Message {
        content: response_text,
        role: "assistant".to_string(),
    })
}

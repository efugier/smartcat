use crate::config::prompt::Message;
use serde::Deserialize;
use std::fmt::Debug;
use std::io;

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

pub(super) fn parse_response(
    response: Result<ureq::Response, ureq::Error>,
) -> io::Result<ureq::Response> {
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

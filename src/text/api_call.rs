use std::time::Duration;

use super::request_schemas::{AnthropicPrompt, OpenAiPrompt};
use super::response_schemas::{AnthropicResponse, OllamaResponse, OpenAiResponse};

use crate::config::{
    api::{Api, ApiConfig},
    prompt::{Message, Prompt},
};
use crate::utils::handle_api_response;

use log::debug;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum PromptFormat {
    OpenAi(OpenAiPrompt),
    Anthropic(AnthropicPrompt),
}

pub fn post_prompt_and_get_answer(
    api_config: ApiConfig,
    prompt: &Prompt,
) -> reqwest::Result<Message> {
    debug!(
        "Trying to reach {:?} with key {:?}",
        api_config.url, api_config.api_key
    );
    debug!("Prompt: {:?}", prompt);

    let mut prompt = prompt.clone();

    if prompt.model.is_none() {
        prompt.model = api_config.default_model.clone()
    }

    // currently not compatible with streams
    prompt.stream = Some(false);

    let client = reqwest::blocking::Client::builder()
        .timeout(
            api_config
                .timeout_seconds
                .map(|t| Duration::from_secs(t.into())),
        )
        .build()
        .expect("Unable to initialize HTTP client");

    let prompt_format = match prompt.api {
        Api::Ollama | Api::Openai | Api::Mistral | Api::Groq => {
            PromptFormat::OpenAi(OpenAiPrompt::from(prompt.clone()))
        }
        Api::Anthropic => PromptFormat::Anthropic(AnthropicPrompt::from(prompt.clone())),
        Api::AnotherApiForTests => panic!("This api is not made for actual use."),
    };

    let request = client
        .post(&api_config.url)
        .header("Content-Type", "application/json")
        .json(&prompt_format);

    // Add auth if necessary
    let request = match prompt.api {
        Api::Openai | Api::Mistral | Api::Groq => request.header(
            "Authorization",
            &format!("Bearer {}", &api_config.get_api_key()),
        ),
        Api::Anthropic => request
            .header("x-api-key", &api_config.get_api_key())
            .header(
                "anthropic-version",
                &api_config.version.expect(
                    "version required for Anthropic, please add version key to your api config",
                ),
            ),
        _ => request,
    };

    let response_text: String = match prompt.api {
        Api::Ollama => handle_api_response::<OllamaResponse>(request.send()?),
        Api::Openai | Api::Mistral | Api::Groq => {
            handle_api_response::<OpenAiResponse>(request.send()?)
        }
        Api::Anthropic => handle_api_response::<AnthropicResponse>(request.send()?),
        Api::AnotherApiForTests => unreachable!(),
    };
    Ok(Message::assistant(&response_text))
}

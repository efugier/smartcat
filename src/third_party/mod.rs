mod prompt_adapters;
mod response_parsing;

use self::prompt_adapters::{AnthropicPrompt, OpenAiPrompt};
use self::response_parsing::{AnthropicResponse, OllamaResponse, OpenAiResponse};
use crate::input_processing::is_interactive;
use crate::{
    config::{
        api::{Api, ApiConfig},
        prompt::{Message, Prompt},
    },
    input_processing::read_user_input,
};

use log::debug;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum PromptFormat {
    OpenAi(OpenAiPrompt),
    Anthropic(AnthropicPrompt),
}

pub fn make_api_request(api_config: ApiConfig, prompt: &Prompt) -> reqwest::Result<Message> {
    debug!(
        "Trying to reach {:?} with key {:?}",
        api_config.url, api_config.api_key
    );
    debug!("Prompt: {:?}", prompt);
    validate_prompt_size(prompt);

    let mut prompt = prompt.clone();

    if prompt.model.is_none() {
        prompt.model = api_config.default_model.clone()
    }

    // currently not compatible with streams
    prompt.stream = Some(false);

    let client = reqwest::blocking::Client::new();

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

/// clean error management
pub fn handle_api_response<T: serde::de::DeserializeOwned + Into<String>>(
    response: reqwest::blocking::Response,
) -> String {
    let status = response.status();
    if response.status().is_success() {
        response.json::<T>().unwrap().into()
    } else {
        let error_text = response.text().unwrap();
        panic!("API request failed with status {}: {}", status, error_text);
    }
}

fn validate_prompt_size(prompt: &Prompt) {
    let char_limit = prompt.char_limit.unwrap_or_default();
    let number_of_chars: u32 = prompt
        .messages
        .iter()
        .map(|message| message.content.len() as u32)
        .sum();

    debug!("Number of chars is prompt: {}", number_of_chars);

    if char_limit > 0 && number_of_chars > char_limit {
        if is_interactive() {
            println!(
                "The number of chars in the input {} is greater than the set limit {}\n\
                Do you want to continue? High costs may ensue.\n[Y/n]",
                number_of_chars, char_limit,
            );
            let input = read_user_input();
            if input.trim() != "Y" {
                println!("exiting...");
                std::process::exit(0);
            }
        } else {
            panic!(
                "Input {} larger than limit {} in non-interactive mode. Exiting.",
                number_of_chars, char_limit
            );
        }
    }
}

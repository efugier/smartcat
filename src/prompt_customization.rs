use glob::glob;
use log::debug;
use std::fs;

use crate::{
    config::{
        prompt::{Message, Prompt},
        PLACEHOLDER_TOKEN,
    },
    PromptParams,
};

pub fn customize_prompt(
    mut prompt: Prompt,
    prompt_params: &PromptParams,
    custom_prompt: Option<String>,
) -> Prompt {
    debug!("pre-customization prompt {:?}", prompt);

    // Override parameters
    if let Some(api) = prompt_params.api.clone() {
        prompt.api = api.to_owned();
    }
    if prompt_params.model.is_some() {
        prompt.model = prompt_params.model.to_owned();
    }
    if prompt_params.char_limit.is_some() {
        prompt.char_limit = prompt_params.char_limit;
    }

    // Collect the content of all the context files
    let context = prompt_params
        .context
        .iter()
        .flat_map(|glob_pattern| {
            glob(glob_pattern)
                .expect("Failed to read glob pattern")
                .filter_map(Result::ok)
                .map(|path| {
                    fs::read_to_string(&path)
                        .ok()
                        .map(|content| format!("{}:\n```\n{}\n```\n", path.display(), content))
                })
        })
        .flatten()
        .collect::<String>();

    if !context.is_empty() {
        prompt.messages.push(Message::system(&format!(
            "files content for context:\n\n{}",
            context
        )));
    }

    // if prompt customization was provided, add it in a new message
    if let Some(command_text) = custom_prompt.clone() {
        let mut prompt_message = command_text;
        if !prompt_message.contains(PLACEHOLDER_TOKEN) {
            prompt_message.push_str(PLACEHOLDER_TOKEN);
        }
        // remove existing input placeholder in order to get just one
        for message in prompt.messages.iter_mut() {
            message.content = message.content.replace(PLACEHOLDER_TOKEN, "");
        }
        prompt.messages.push(Message::user(&prompt_message));
    }

    // get the last message for check and make sure it's a user one
    let mut last_message =
        if prompt.messages.is_empty() || prompt.messages.last().is_some_and(|m| m.role != "user") {
            Message::user(PLACEHOLDER_TOKEN)
        } else {
            prompt.messages.pop().unwrap()
        };

    // verify that the last message contrains a placeholder
    if !last_message.content.contains(PLACEHOLDER_TOKEN) {
        last_message.content.push_str(PLACEHOLDER_TOKEN);
    }

    if let Some(temperature) = prompt_params.temperature {
        if temperature == 0. {
            // a temperature of 0 does not lead to a deterministic result for current API
            prompt.temperature = Some(1e-13);
        } else {
            prompt.temperature = prompt_params.temperature;
        }
    }
    prompt.messages.push(last_message);

    debug!("post-customization prompt {:?}", prompt);

    prompt
}

#[cfg(test)]
mod tests {
    use crate::config::api::Api;
    use std::io::Write;

    use super::*;

    #[test]
    fn test_customize_prompt_empty_no_overrides() {
        let prompt = Prompt::default();
        let prompt_params = PromptParams::default();

        let customized = customize_prompt(prompt, &prompt_params, None);
        let default_prompt = Prompt::empty();

        assert_eq!(customized.api, default_prompt.api);
        assert_eq!(customized.model, default_prompt.model);
        assert_eq!(customized.temperature, default_prompt.temperature);
        assert_eq!(
            customized.messages,
            vec![
                Prompt::default().messages.first().unwrap().to_owned(),
                Message::user(PLACEHOLDER_TOKEN)
            ]
        );

        // check placeholder existance
        assert!(
            customized
                .messages
                .last()
                .unwrap()
                .content
                .contains(PLACEHOLDER_TOKEN),
            "The last message should contain the placeholder."
        );
    }

    #[test]
    fn test_customize_prompt_api_override() {
        let prompt = Prompt::empty();
        let prompt_params = PromptParams {
            api: Some(Api::AnotherApiForTests),
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);
        let default_prompt = Prompt::empty();

        assert_eq!(customized.api, Api::AnotherApiForTests);
        assert_eq!(customized.model, default_prompt.model);
    }

    #[test]
    fn test_customize_prompt_model_override() {
        let prompt = Prompt::empty();
        let prompt_params = PromptParams {
            model: Some("test_model".to_owned()),
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);

        let default_prompt = Prompt::empty();
        assert_eq!(customized.model, prompt_params.model);
        assert_eq!(customized.api, default_prompt.api);
    }

    #[test]
    fn test_customize_prompt_command_insertion() {
        let prompt = Prompt::empty();
        let prompt_params = PromptParams::default();
        let custom_prompt = Some("test_command".to_owned());

        let customized = customize_prompt(prompt, &prompt_params, custom_prompt);

        assert!(customized
            .messages
            .iter()
            .any(|m| m.content.contains("test_command")));
    }

    #[test]
    fn test_customize_prompt_with_context_file() {
        let prompt = Prompt::empty();
        let context_content = "hello there".to_owned();
        let mut context_file = tempfile::NamedTempFile::new().unwrap();
        context_file.write_all(context_content.as_bytes()).unwrap();

        let prompt_params = PromptParams {
            context: vec![context_file.path().to_str().unwrap().to_owned()],
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);

        assert_eq!(
            customized.messages[0].content,
            format!(
                "files content for context:\n\n{}:\n```\n{}\n```\n",
                context_file.path().display(),
                context_content
            )
        );
        assert_eq!(customized.messages[0].role, "system");
    }

    #[test]
    fn test_customize_prompt_temperature_override() {
        let prompt = Prompt::empty();
        let prompt_params = PromptParams {
            temperature: Some(42.),
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);

        assert_eq!(customized.temperature, Some(42.));
    }

    #[test]
    fn test_customize_prompt_with_all_overrides() {
        let prompt = Prompt::empty();
        let context_content = "hello there".to_owned();
        let mut context_file = tempfile::NamedTempFile::new().unwrap();
        context_file.write_all(context_content.as_bytes()).unwrap();

        let prompt_params = PromptParams {
            api: Some(Api::AnotherApiForTests),
            model: Some("test_model_override".to_owned()),
            context: vec![context_file.path().to_str().unwrap().to_owned()],
            temperature: Some(42.),
            char_limit: Some(50_000),
        };
        let custom_prompt = Some("test_command_override".to_owned());

        let customized = customize_prompt(prompt, &prompt_params, custom_prompt.clone());

        // Mandatory fields
        assert_eq!(customized.api, prompt_params.api.unwrap());
        assert!(customized
            .messages
            .iter()
            .any(|m| m.content.contains(custom_prompt.as_ref().unwrap())));

        // Optional fields
        assert_eq!(customized.model, prompt_params.model);
        assert_eq!(customized.temperature, prompt_params.temperature);
        assert_eq!(
            customized.messages[0].content,
            format!(
                "files content for context:\n\n{}:\n```\n{}\n```\n",
                context_file.path().display(),
                context_content
            )
        );
        assert_eq!(customized.messages[0].role, "system");
    }
}

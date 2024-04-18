use glob::glob;
use log::debug;
use std::fs;

use crate::{
    config::{Message, Prompt, PLACEHOLDER_TOKEN},
    PromptParams,
};

// TODO: simplify this mess
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

    let mut first_user_message_index = prompt
        .messages
        .iter()
        .position(|m| m.role == "user")
        .unwrap_or(0);

    // insert system or context messages
    let mut maybe_insert_message = |content: Option<String>, prefix: Option<String>| {
        if let Some(mut content) = content {
            if let Some(mut pre) = prefix {
                pre.push_str(&content);
                content = pre;
            }

            let system_message = Message::system(&content);
            prompt
                .messages
                .insert(first_user_message_index, system_message);
            first_user_message_index += 1;
        }
    };
    maybe_insert_message(prompt_params.system_message.clone(), None);

    // insert matched file's content as context in a system message
    let context = prompt_params.context.clone().and_then(|glob_pattern| {
        let files_content = glob(&glob_pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .filter_map(|path| {
                fs::read_to_string(&path)
                    .ok()
                    .map(|content| format!("{}:\n```\n{}\n```\n", path.display(), content))
            })
            .collect::<String>();

        (!files_content.is_empty()).then_some(files_content)
    });
    maybe_insert_message(context, Some("files content for context:\n\n".to_owned()));

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
        if prompt.messages.is_empty() | prompt.messages.last().is_some_and(|m| m.role != "user") {
            Message::user(PLACEHOLDER_TOKEN)
        } else {
            prompt.messages.pop().unwrap()
        };

    // verify that the last message contrains a placeholder
    if !last_message.content.contains(PLACEHOLDER_TOKEN) {
        last_message.content.push_str(PLACEHOLDER_TOKEN);
    }

    // add the after input text
    if let Some(after_input_text) = prompt_params.after_input.clone() {
        last_message.content.push_str(&after_input_text);
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
    use crate::config::Api;
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
    fn test_customize_prompt_system_message_insertion() {
        let prompt = Prompt::empty();
        let prompt_params = PromptParams {
            system_message: Some("system message".to_owned()),
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);

        assert_eq!(
            customized.messages[0].content,
            prompt_params.system_message.unwrap(),
            "{:?}",
            customized.messages
        );
        assert_eq!(
            customized.messages[0].role,
            Message::system("").role,
            "{:?}",
            customized.messages
        );
    }

    #[test]
    fn test_customize_prompt_system_message_insertion_with_user_message() {
        let mut prompt = Prompt::empty();
        prompt.messages.push(Message::user("user message"));
        let prompt_params = PromptParams {
            system_message: Some("system message".to_owned()),
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);

        assert_eq!(
            customized.messages[0].content,
            prompt_params.system_message.unwrap(),
            "{:?}",
            customized.messages
        );
        assert_eq!(
            customized.messages[0].role,
            Message::system("").role,
            "{:?}",
            customized.messages
        );
    }

    #[test]
    fn test_customize_prompt_with_context_file() {
        let prompt = Prompt::empty();
        let context_content = "hello there".to_owned();
        let mut context_file = tempfile::NamedTempFile::new().unwrap();
        context_file.write_all(context_content.as_bytes()).unwrap();

        let prompt_params = PromptParams {
            context: Some(context_file.path().to_str().unwrap().to_owned()),
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
    fn test_customize_prompt_after_input_insertion() {
        let mut prompt = Prompt::empty();
        // Adding placeholder and command to ensure they are in the last user message.
        prompt
            .messages
            .push(Message::user(&format!("command {}", PLACEHOLDER_TOKEN)));

        let prompt_params = PromptParams {
            after_input: Some("-- after input".to_owned()),
            ..PromptParams::default()
        };

        let customized = customize_prompt(prompt, &prompt_params, None);

        let last_message_content = &customized.messages.last().unwrap().content;
        assert!(
            last_message_content.ends_with("-- after input"),
            "The last message should end with the after input text. Got {}",
            &last_message_content
        )
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
            context: Some(context_file.path().to_str().unwrap().to_owned()),
            after_input: Some(" test_after_input_override".to_owned()),
            system_message: Some("system message override".to_owned()),
            temperature: Some(42.),
        };
        let custom_prompt = Some("test_command_override".to_owned());

        let customized = customize_prompt(prompt, &prompt_params, custom_prompt.clone());

        // Mandatory fields
        assert_eq!(customized.api, prompt_params.api.unwrap());
        assert!(customized
            .messages
            .iter()
            .any(|m| m.content.contains(custom_prompt.as_ref().unwrap())));
        assert_eq!(
            customized.messages[0].content,
            prompt_params.system_message.unwrap()
        );
        assert_eq!(customized.messages[0].role, "system");

        // Optional fields
        assert_eq!(customized.model, prompt_params.model);
        assert_eq!(customized.temperature, prompt_params.temperature);
        assert_eq!(
            customized.messages[1].content,
            format!(
                "files content for context:\n\n{}:\n```\n{}\n```\n",
                context_file.path().display(),
                context_content
            )
        );
        assert_eq!(customized.messages[1].role, "system");
        assert!(
            customized
                .messages
                .last()
                .unwrap()
                .content
                .ends_with(&prompt_params.after_input.unwrap()),
            "The last message should end with the after input text."
        );
    }
}

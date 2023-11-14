use log::debug;
use std::fs;

use crate::config::{Api, Message, Prompt, PLACEHOLDER_TOKEN};

pub fn customize_prompt(
    mut prompt: Prompt,
    api: &Option<Api>,
    model: &Option<String>,
    command: &Option<String>,
    after_input: &Option<String>,
    system_message: Option<String>,
    context: Option<String>,
) -> Prompt {
    debug!("pre-customization prompt {:?}", prompt);
    // Override parameters
    if let Some(api) = api {
        prompt.api = api.to_owned();
    }
    if let Some(model) = model {
        prompt.model = model.to_owned();
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
    maybe_insert_message(system_message, None);
    // context can be a file
    let context = context.map(|ctx| fs::read_to_string(&ctx).unwrap_or(ctx));
    maybe_insert_message(context, Some("context:\n".to_string()));

    // if prompt customization was provided, add it in a new message
    if let Some(command_text) = command {
        let mut prompt_message = String::from(command_text);
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
    if let Some(after_input_text) = after_input {
        last_message.content.push_str(after_input_text);
    }

    prompt.messages.push(last_message);

    debug!("post-customization prompt {:?}", prompt);

    prompt
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_customize_prompt_empty_no_overrides() {
        let prompt = Prompt::empty();

        let customized = customize_prompt(prompt, &None, &None, &None, &None, None, None);
        let default_prompt = Prompt::empty();

        assert_eq!(customized.api, default_prompt.api);
        assert_eq!(customized.model, default_prompt.model);
        assert_eq!(customized.messages, vec![Message::user(PLACEHOLDER_TOKEN)]);
    }

    #[test]
    fn test_customize_prompt_api_override() {
        let prompt = Prompt::empty();
        let api = Api::AnotherApiForTests;

        let customized =
            customize_prompt(prompt, &Some(api.clone()), &None, &None, &None, None, None);
        let default_prompt = Prompt::empty();

        assert_eq!(customized.api, Api::AnotherApiForTests);
        assert_eq!(customized.model, default_prompt.model);
    }

    #[test]
    fn test_customize_prompt_model_override() {
        let prompt = Prompt::empty();
        let model = "test_model".to_owned();

        let customized = customize_prompt(
            prompt,
            &None,
            &Some(model.clone()),
            &None,
            &None,
            None,
            None,
        );

        let default_prompt = Prompt::empty();
        assert_eq!(customized.model, model);
        assert_eq!(customized.api, default_prompt.api);
    }

    #[test]
    fn test_customize_prompt_command_insertion() {
        let prompt = Prompt::empty();
        let command = "test_command".to_owned();

        let customized = customize_prompt(
            prompt,
            &None,
            &None,
            &Some(command.clone()),
            &None,
            None,
            None,
        );

        assert!(customized
            .messages
            .iter()
            .any(|m| m.content.contains(&command)));
    }

    #[test]
    fn test_customize_prompt_system_message_insertion() {
        let prompt = Prompt::empty();
        let system_message = "system message".to_owned();

        let customized = customize_prompt(
            prompt,
            &None,
            &None,
            &None,
            &None,
            Some(system_message.clone()),
            None,
        );

        assert_eq!(
            customized.messages[0].content, system_message,
            "{:?}",
            customized.messages
        );
        assert_eq!(
            customized.messages[0].role, "system",
            "{:?}",
            customized.messages
        );
    }

    #[test]
    fn test_customize_prompt_system_message_insertion_with_user_message() {
        let mut prompt = Prompt::empty();
        prompt.messages.push(Message::user("user message"));
        let system_message = "system message".to_owned();

        let customized = customize_prompt(
            prompt,
            &None,
            &None,
            &None,
            &None,
            Some(system_message.clone()),
            None,
        );

        assert_eq!(
            customized.messages[1].content, system_message,
            "{:?}",
            customized.messages
        );
        assert_eq!(
            customized.messages[1].role, "system",
            "{:?}",
            customized.messages
        );
    }

    #[test]
    fn test_customize_prompt_after_input_insertion() {
        let mut prompt = Prompt::empty();
        let after_input = " after input".to_owned();
        // Adding placeholder and command to ensure they are in the last user message.
        prompt
            .messages
            .push(Message::user(&format!("command {}", PLACEHOLDER_TOKEN)));

        let customized = customize_prompt(
            prompt,
            &None,
            &None,
            &None,
            &Some(after_input.clone()),
            None,
            None,
        );

        let last_message_content = &customized.messages.last().unwrap().content;
        assert!(
            last_message_content.ends_with(&after_input),
            "The last message should end with the after input text. Got {}",
            &last_message_content
        )
    }

    #[test]
    fn test_customize_prompt_placeholder_existence() {
        let prompt = Prompt::empty();

        let customized = customize_prompt(prompt, &None, &None, &None, &None, None, None);

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
    fn test_customize_prompt_with_all_overrides() {
        let prompt = Prompt::empty();
        let api = Api::AnotherApiForTests;
        let model = "test_model_override".to_owned();
        let command = "test_command_override".to_owned();
        let after_input = " test_after_input_override".to_owned();
        let system_message = "system message override".to_owned();
        let mut context_file = tempfile::NamedTempFile::new().unwrap();

        context_file.write_all("hello there".as_bytes());

        let customized = customize_prompt(
            prompt,
            &Some(api.clone()),
            &Some(model.clone()),
            &Some(command.clone()),
            &Some(after_input.clone()),
            Some(system_message.clone()),
            Some(context_file.path().to_str().unwrap().to_owned()),
        );

        assert_eq!(customized.api, api);
        assert_eq!(customized.model, model);
        assert!(customized
            .messages
            .iter()
            .any(|m| m.content.contains(&command)));
        assert_eq!(customized.messages[0].content, system_message);
        assert_eq!(customized.messages[0].role, "system");
        assert!(
            customized
                .messages
                .last()
                .unwrap()
                .content
                .ends_with(&after_input),
            "The last message should end with the after input text."
        );
    }
}

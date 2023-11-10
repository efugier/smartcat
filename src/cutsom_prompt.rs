use log::debug;

use crate::config::{Message, Prompt, PLACEHOLDER_TOKEN};

pub fn customize_prompt(
    mut prompt: Prompt,
    api: &Option<String>,
    model: &Option<String>,
    command: &Option<String>,
    after_input: &Option<String>,
    system_message: &Option<String>,
) -> Prompt {
    debug!("pre-customization promot {:?}", prompt);
    // Override parameters
    if let Some(api) = api {
        prompt.api = api.to_owned();
    }
    if let Some(model) = model {
        prompt.model = model.to_owned();
    }

    let first_user_message_index = prompt.messages.iter().position(|m| m.role == "system");

    // if there's a system message to add, add it before the first user message
    if let Some(message_content) = system_message {
        let system_message = Message {
            role: "system".to_string(),
            content: message_content.to_owned(),
        };
        if let Some(index) = first_user_message_index {
            prompt.messages.insert(index, system_message);
        } else {
            prompt.messages.push(system_message);
        }
    }

    // if prompt customization was provided, add it in a new message
    let mut prompt_message = String::new();
    if let Some(command_text) = command {
        prompt_message.push_str(command_text);
        if !prompt_message.contains(PLACEHOLDER_TOKEN) {
            prompt_message.push_str(PLACEHOLDER_TOKEN);
        }
    }
    if !prompt_message.is_empty() {
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: prompt_message,
        });
    }

    // get the last message for check and make sure it's a user one
    let mut last_message =
        if prompt.messages.is_empty() | prompt.messages.last().is_some_and(|m| m.role != "user") {
            Message {
                role: "user".to_string(),
                content: PLACEHOLDER_TOKEN.to_string(),
            }
        } else {
            prompt.messages.pop().unwrap()
        };

    // verify that the last message contrains a placeholder
    if !last_message.content.contains(PLACEHOLDER_TOKEN) {
        last_message.content.push_str(PLACEHOLDER_TOKEN);
    }

    // add the after input text
    if let Some(after_input_text) = after_input {
        let last_message = prompt.messages.last_mut().unwrap();
        last_message.content.push_str(after_input_text);
    }

    prompt.messages.push(last_message);

    debug!("pre-customization promot {:?}", prompt);
    prompt
}

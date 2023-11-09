use log::debug;

use crate::config::{Message, Prompt, PLACEHOLDER_TOKEN};

pub fn customize_prompt(
    mut prompt: Prompt,
    command: &Option<String>,
    after: &Option<String>,
    system_message: &Option<String>,
) -> Prompt {
    debug!("test");
    let empty_prompt = prompt.messages.is_empty();

    // if there's a system message to add, add it before the first user message
    if let Some(message_content) = system_message {
        let system_message = Message {
            role: "system".to_string(),
            content: message_content.to_owned(),
        };

        let first_user_message_index = prompt.messages.iter().position(|m| m.role == "system");
        if let Some(index) = first_user_message_index {
            prompt.messages.insert(index, system_message);
        } else {
            prompt.messages.push(system_message);
        }
    }

    // add stuff if there's some custom things to do
    let mut prompt_message = String::new();
    if let Some(command_text) = command {
        prompt_message.push_str(command_text);
        if !prompt_message.contains(PLACEHOLDER_TOKEN) {
            prompt_message.push_str(PLACEHOLDER_TOKEN);
        }
    }
    if let Some(after_input) = after {
        prompt_message.push_str(after_input);
    }

    let last_message_contains_input = prompt
        .messages
        .last()
        .is_some_and(|m| m.content.contains(PLACEHOLDER_TOKEN));

    if !prompt_message.is_empty() {
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: prompt_message,
        });
    } else if last_message_contains_input {
        // no command and an empty prompt -> use input as prompt
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: PLACEHOLDER_TOKEN.to_string(),
        });
    }

    if empty_prompt {
        // no command and an empty prompt -> use input as prompt
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: PLACEHOLDER_TOKEN.to_string(),
        });
    }
    prompt
}

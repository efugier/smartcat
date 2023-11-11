use log::debug;

use crate::config::{Api, Message, Prompt, PLACEHOLDER_TOKEN};

pub fn customize_prompt(
    mut prompt: Prompt,
    api: &Option<Api>,
    model: &Option<String>,
    command: &Option<String>,
    after_input: &Option<String>,
    system_message: &Option<String>,
) -> Prompt {
    debug!("pre-customization prompt {:?}", prompt);
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
        let system_message = Message::system(message_content);
        if let Some(index) = first_user_message_index {
            prompt.messages.insert(index, system_message);
        } else {
            prompt.messages.push(system_message);
        }
    }

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
        let last_message = prompt.messages.last_mut().unwrap();
        last_message.content.push_str(after_input_text);
    }

    prompt.messages.push(last_message);

    debug!("post-customization prompt {:?}", prompt);

    prompt
}

use log::debug;

use crate::config::{Message, Prompt, PLACEHOLDER_TOKEN};

pub fn customize_prompt(
    mut prompt: Prompt,
    command: &Option<String>,
    before: &Option<String>,
    after: &Option<String>,
    system_message: &Option<String>,
) -> Prompt {
    debug!("test");
    let empty_prompt = prompt.messages.is_empty();

    if let Some(message_content) = system_message {
        prompt.messages.push(Message {
            role: "system".to_string(),
            content: message_content.to_owned(),
        });
    }
    if command.is_some() {
        let mut prompt_message: String = [before, command, after]
            .into_iter()
            .filter_map(|x| x.to_owned())
            .collect();
        prompt_message.push_str(PLACEHOLDER_TOKEN);
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: prompt_message,
        });
    } else if empty_prompt {
        // no command and an empty prompt -> use input as prompt
        prompt.messages.push(Message {
            role: "user".to_string(),
            content: PLACEHOLDER_TOKEN.to_string(),
        });
    }
    prompt
}

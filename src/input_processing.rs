use log::debug;
use std::io::{Result, Write};

use crate::config::{api::get_api_config, prompt::Prompt, PLACEHOLDER_TOKEN};
use crate::third_party::make_api_request;

pub const IS_NONINTERACTIVE_ENV_VAR: &str = "SMARTCAT_NONINTERACTIVE";

pub fn is_interactive() -> bool {
    std::env::var(IS_NONINTERACTIVE_ENV_VAR).unwrap_or_default() != "1"
}

pub fn read_user_input() -> String {
    let mut user_input = String::new();
    std::io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");
    user_input.trim().to_string()
}

pub fn process_input_with_request<W: Write>(
    mut prompt: Prompt,
    mut input: String,
    output: &mut W,
    repeat_input: bool,
) -> Result<Prompt> {
    // insert the input in the messages with placeholders
    for message in prompt.messages.iter_mut() {
        message.content = message.content.replace(PLACEHOLDER_TOKEN, &input)
    }
    // fetch the api config tied to the prompt
    let api_config = get_api_config(&prompt.api.to_string());

    let response_message = match make_api_request(api_config, &prompt) {
        Ok(message) => message,
        Err(e) => {
            eprintln!("Failed to make API request: {:?}", e);
            std::process::exit(1);
        }
    };
    debug!("{}", &response_message.content);

    prompt.messages.push(response_message.clone());

    if repeat_input {
        input.push('\n');
        output.write_all(input.as_bytes())?;
    }

    output.write_all(response_message.content.as_bytes())?;

    Ok(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_process_input {
        ($test_name:ident, $prefix:expr, $suffix:expr, $input:expr) => {
            #[test]
            fn $test_name() {
                let mut output = std::io::Cursor::new(Vec::new());

                let result =
                    output.write_all(format!("{}{}{}", $prefix, $input, $suffix).as_bytes());
                assert!(result.is_ok());

                let expected_output = if !$input.is_empty() {
                    format!("{}{}{}", $prefix, $input, $suffix)
                } else {
                    "".into()
                };

                let expected_output_as_bytes = expected_output.as_bytes();
                let output_data: Vec<u8> = output.into_inner();
                assert_eq!(
                    expected_output_as_bytes,
                    output_data,
                    "\nexpected: {}\nGot: {}",
                    String::from_utf8_lossy(expected_output_as_bytes),
                    &expected_output
                );
            }
        };
    }

    test_process_input!(
        test_with_prefix_and_suffix,
        "Prefix: ",
        " Suffix",
        "Input data"
    );
    test_process_input!(
        test_with_custom_prefix_suffix,
        "Start: ",
        " End",
        "Custom input"
    );
}

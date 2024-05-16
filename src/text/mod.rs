mod api_call;
mod request_schemas;
mod response_schemas;

use log::debug;
use std::io::{Result, Write};

use self::api_call::post_prompt_and_get_answer;
use crate::config::{api::get_api_config, prompt::Prompt, PLACEHOLDER_TOKEN};
use crate::utils::{is_interactive, read_user_input};

/// insert the input in the prompt, validate the length and make the request
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

    validate_prompt_size(&prompt);
    let response_message = match post_prompt_and_get_answer(api_config, &prompt) {
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

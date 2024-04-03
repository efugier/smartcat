use log::debug;
use std::io::{Read, Result, Write};

use crate::config::{get_api_config, Prompt, PLACEHOLDER_TOKEN};
use crate::request::make_api_request;

// [tmp] mostly template to write tests
pub fn chunk_process_input<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
    prefix: &str,
    suffix: &str,
) -> Result<()> {
    let mut first_chunk = true;
    let mut buffer = [0; 1024];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => break, // end of input
            Ok(n) => {
                if first_chunk {
                    output.write_all(prefix.as_bytes())?;
                    first_chunk = false;
                }
                output.write_all(&buffer[..n])?;
            }
            Err(e) => return Err(e),
        }
    }

    if !first_chunk {
        // we actually got some input
        output.write_all(suffix.as_bytes())?;
    }

    Ok(())
}

pub fn process_input_with_request<R: Read, W: Write>(
    mut prompt: Prompt,
    input: &mut R,
    input_string: Option<String>,
    output: &mut W,
    repeat_input: bool,
) -> Result<Prompt> {
    let mut input = match input_string {
        Some(input_string) => input_string,
        None => {
            let mut buffer = Vec::new();
            input.read_to_end(&mut buffer)?;

            String::from_utf8(buffer).unwrap()
        }
    };

    // nothing to do if no input
    if input.is_empty() {
        return Ok(prompt);
    }

    // insert the input in the messages with placeholders
    for message in prompt.messages.iter_mut() {
        message.content = message.content.replace(PLACEHOLDER_TOKEN, &input)
    }
    // fetch the api config tied to the prompt
    let api_config = get_api_config(&prompt.api.to_string());

    // make the request
    let response_message = make_api_request(api_config, &prompt)?;

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
    use std::io::Cursor;

    macro_rules! test_process_input {
        ($test_name:ident, $prefix:expr, $suffix:expr, $input:expr) => {
            #[test]
            fn $test_name() {
                let input = $input.as_bytes();
                let mut output = std::io::Cursor::new(Vec::new());

                let result =
                    chunk_process_input(&mut Cursor::new(input), &mut output, $prefix, $suffix);
                assert!(result.is_ok());

                let expected_output = if !input.is_empty() {
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
    test_process_input!(test_empty_input, "Pre: ", " Post", "");
}

mod input_processing;
mod prompt_customization;
mod request;

#[allow(dead_code)]
mod config;

use clap::{Args, Parser};
use log::debug;
use std::collections::HashMap;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};

use crate::config::get_last_conversation_as_prompt;
use prompt_customization::customize_prompt;

#[derive(Debug, Parser)]
#[command(
    name = "smartcat (sc)",
    author = "Emilien Fugier",
    version = "0.7.4",
    about = "Putting a brain behind `cat`. CLI interface to bring language models in the Unix ecosystem 🐈‍⬛",
    long_about = None
)]
struct Cli {
    #[arg(default_value_t = String::from("default"))]
    /// an input or which prompt in the config to use
    input_or_config_prompt: String,
    /// the input if the first arg a config prompt
    input_if_config_prompt: Option<String>,
    /// whether to extend the previous conversation or start a new one
    #[arg(short, long)]
    extend_conversation: bool,
    /// whether to repeat the input before the output, useful to extend instead of replacing
    #[arg(short, long)]
    repeat_input: bool,
    #[command(flatten)]
    prompt_params: PromptParams,
}

#[derive(Debug, Default, Args)]
#[group(id = "prompt_params")]
struct PromptParams {
    /// overrides which api to hit
    #[arg(long)]
    api: Option<config::Api>,
    /// overrides which model (of the api) to use
    #[arg(short, long)]
    model: Option<String>,
    /// suffix to add after the input and the custom prompt
    #[arg(short, long)]
    after_input: Option<String>,
    /// system "config"  message to send after the prompt and before the first user message
    #[arg(short, long)]
    system_message: Option<String>,
    /// glob pattern to given the matched files' content as context
    #[arg(short, long)]
    context: Option<String>,
    /// temperature between 0 and 2, higher means answer further from the average
    #[arg(short, long)]
    temperature: Option<f32>,
}

fn main() {
    env_logger::init();

    let stdin = io::stdin();
    let mut output = io::stdout();

    // case for testing
    // TODO: mock API and actually the real processing
    if std::env::var("SMARTCAT_TEST").unwrap_or_default() == "1" {
        let prefix = String::from("Hello, World!\n```\n");
        let suffix = String::from("\n```\n");
        let mut input = String::new();

        if let Err(e) = stdin
            .lock()
            .read_to_string(&mut input)
            .and(output.write_all(format!("{}{}{}", prefix, input, suffix).as_bytes()))
        {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    let args = Cli::parse();

    config::ensure_config_files(true)
        .expect("Unable to verify that the config files exist or to generate new ones.");

    let mut input = String::new();

    let is_piped = !stdin.is_terminal();
    if is_piped {
        stdin.lock().read_to_string(&mut input).unwrap();
    }

    let mut custom_prompt: Option<String> = None;
    let prompt: config::Prompt = if args.extend_conversation {
        get_last_conversation_as_prompt()
    } else {
        let mut prompts = config::get_prompts();
        get_default_and_or_custom_prompt(&args, &mut prompts, &mut custom_prompt)
    };

    // if no text was piped, use the custom prompt as input
    if input.is_empty() {
        input.push_str(&custom_prompt.unwrap_or_default());
        custom_prompt = None;
    }

    let prompt = customize_prompt(prompt, &args.prompt_params, custom_prompt);

    debug!("{:?}", prompt);

    match input_processing::process_input_with_request(
        prompt,
        input,
        &mut output,
        args.repeat_input,
    ) {
        Ok(prompt) => {
            let toml_string = toml::to_string(&prompt).expect("Failed to serialize prompt");
            let mut file = fs::File::create(config::conversation_file_path())
                .expect("Failed to the conversation save file");
            file.write_all(toml_string.as_bytes())
                .expect("Failed to write to file");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_default_and_or_custom_prompt(
    args: &Cli,
    prompts: &mut HashMap<String, config::Prompt>,
    custom_prompt: &mut Option<String>,
) -> config::Prompt {
    let available_prompts: Vec<&String> = prompts.keys().collect();
    let prompt_not_found_error = format!(
        "`default` prompt not found, available ones are: {:?}",
        &available_prompts
    );

    if let Some(prompt) = prompts.remove(&args.input_or_config_prompt) {
        if let Some(text) = args.input_if_config_prompt.clone() {
            *custom_prompt = Some(text);
        }
        prompt
    } else {
        *custom_prompt = Some(String::from(&args.input_or_config_prompt));
        if args.input_if_config_prompt.is_some() {
            panic!("Invalid parameter, either provide a valid config prompt then an input, or only an input");
        }
        prompts.remove("default").expect(&prompt_not_found_error)
    }
}

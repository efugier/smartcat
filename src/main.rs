mod input_processing;
mod prompt_customization;
mod third_party;

mod config;

use crate::config::{
    api::Api,
    ensure_config_usable,
    prompt::{conversation_file_path, get_last_conversation_as_prompt, get_prompts, Prompt},
};
use prompt_customization::customize_prompt;

use clap::{Args, Parser};
use log::debug;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};

const DEFAULT_PROMPT_NAME: &str = "default";

#[derive(Debug, Parser)]
#[command(
    name = "smartcat (sc)",
    author = "Emilien Fugier",
    version = "1.2.0",
    about = "Putting a brain behind `cat`. CLI interface to bring language models in the Unix ecosystem 🐈‍⬛",
    long_about = None,
    after_help = "Examples:
=========
- sc \"say hi\"  # just ask

- sc test                         # use templated prompts
- sc test \"and parametrize them\"  # extend them on the fly

- sc \"explain how to use this program\" -c **/*.md main.py  # use files as context

- git diff | sc \"summarize the changes\"  # pipe data in

- cat en.md | sc \"translate in french\" >> fr.md   # write data out
- sc -e \"use a more informal tone\" -t 2 >> fr.md  # extend the conversation and raise the temprature
"
)]
struct Cli {
    /// ref to a prompt from config or straight input (will use `default` prompt template)
    input_or_config_ref: Option<String>,
    /// if the first arg matches a config ref, the second will be used as input
    input_if_config_ref: Option<String>,
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
    api: Option<Api>,
    /// overrides which model (of the api) to use
    #[arg(short, long)]
    model: Option<String>,
    /// temperature higher means answer further from the average
    #[arg(short, long)]
    temperature: Option<f32>,
    /// max number of chars to include, ask for user approval if more, 0 = no limit
    #[arg(short = 'l', long)]
    char_limit: Option<u32>,
    /// glob patterns or list of files to use the content as context
    /// make sure it's the last arg.
    #[arg(short, long, num_args= 1.., value_delimiter = ' ', verbatim_doc_comment)]
    context: Vec<String>,
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

    debug!("args: {:?}", args);

    config::ensure_config_files()
        .expect("Unable to verify that the config files exist or to generate new ones.");

    let mut input = String::new();

    let is_piped = !stdin.is_terminal();
    let mut custom_prompt: Option<String> = None;

    let prompt: Prompt = if args.extend_conversation {
        custom_prompt = args.input_or_config_ref;
        if args.input_if_config_ref.is_some() {
            panic!(
                "Invalid parameters, cannot provide a config ref when extending a conversation.\n\
                Use `sc -e \"<your_prompt>.\"`"
            );
        }
        get_last_conversation_as_prompt()
    } else {
        get_default_and_or_custom_prompt(&args, &mut custom_prompt)
    };

    // if no text was piped, use the custom prompt as input
    if is_piped {
        stdin.lock().read_to_string(&mut input).unwrap();
    }

    if input.is_empty() {
        input.push_str(&custom_prompt.unwrap_or_default());
        custom_prompt = None;
    }

    debug!("input: {}", input);
    debug!("custom_prompt: {:?}", custom_prompt);

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
            let mut file = fs::File::create(conversation_file_path())
                .expect("Failed to the conversation save file");
            file.write_all(toml_string.as_bytes())
                .expect("Failed to write to file");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ensure_config_usable();
            std::process::exit(1);
        }
    }
}

fn get_default_and_or_custom_prompt(args: &Cli, custom_prompt: &mut Option<String>) -> Prompt {
    let mut prompts = get_prompts();
    let available_prompts: Vec<&String> = prompts.keys().collect();
    let prompt_not_found_error = format!(
        "`default` prompt not found, available ones are: {:?}",
        &available_prompts
    );

    let input_or_config_ref = args
        .input_or_config_ref
        .clone()
        .unwrap_or_else(|| String::from("default"));

    if let Some(prompt) = prompts.remove(&input_or_config_ref) {
        if args.input_if_config_ref.is_some() {
            *custom_prompt = args.input_if_config_ref.clone()
        }
        prompt
    } else {
        *custom_prompt = Some(input_or_config_ref);
        if args.input_if_config_ref.is_some() {
            panic!(
                "Invalid parameters, either provide a valid ref to a config prompt then an input, or only an input.\n\
                Use `sc <config_ref> \"<your_prompt\"` or `sc \"<your_prompt>\"`"
            );
        }
        prompts
            .remove(DEFAULT_PROMPT_NAME)
            .expect(&prompt_not_found_error)
    }
}

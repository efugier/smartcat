mod config;
mod prompt_customization;
mod text;
mod utils;
mod voice;

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

use text::process_input_with_request;

const DEFAULT_PROMPT_NAME: &str = "default";

#[derive(Debug, Parser)]
#[command(
    name = "smartcat (sc)",
    author = "Emilien Fugier",
    version = "1.7.2",
    about = "Putting a brain behind `cat`. CLI interface to bring language models in the Unix ecosystem 🐈‍⬛",
    long_about = None,
    after_help = "Examples:
=========

- sc <my_input>
- sc <my_prompt_template_name> <my_input>

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
    /// ref to a prompt template from config or straight input (will use `default` prompt template if input)
    input_or_template_ref: Option<String>,
    /// if the first arg matches a config template, the second will be used as input
    input_if_template_ref: Option<String>,
    /// whether to extend the previous conversation or start a new one
    #[arg(short, long)]
    extend_conversation: bool,
    /// whether to repeat the input before the output, useful to extend instead of replacing
    #[arg(short, long)]
    repeat_input: bool,
    /// whether to use voice for input
    #[arg(short, long)]
    voice: bool,
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
    /// higher temperature  means answer further from the average
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
    let mut input = String::new();

    // case for testing
    // TODO: mock API and actually use the real processing
    if std::env::var("SMARTCAT_TEST").unwrap_or_default() == "1" {
        if let Err(e) = stdin
            .lock()
            .read_to_string(&mut input)
            .and(output.write_all(format!("Hello, World!\n```\n{}\n```\n", input).as_bytes()))
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

    let is_piped = !stdin.is_terminal();
    let mut prompt_customizaton_text: Option<String> = None;

    let prompt: Prompt = if !args.extend_conversation {
        // try to get prompt matching the first arg and use second arg as customization text
        // if it doesn't use default prompt and treat that first arg as customization text
        get_default_and_or_custom_prompt(&args, &mut prompt_customizaton_text)
    } else {
        if args.voice {
            if args.input_or_template_ref.is_some() {
                panic!(
                    "Invalid parameters, when extending conversation and using voice you can't provide additional input args.\n\
                    Use `sc -e -v`"
                );
            }
            prompt_customizaton_text = voice::record_voice_and_get_transcript();
        } else {
            prompt_customizaton_text = args.input_or_template_ref;
        }
        if args.input_if_template_ref.is_some() {
            panic!(
                "Invalid parameters, cannot provide a config ref when extending a conversation.\n\
                Use `sc -e \"<your_prompt>.\"`"
            );
        }
        get_last_conversation_as_prompt()
    };

    // if no text was piped, use the custom prompt as input
    if is_piped {
        stdin.lock().read_to_string(&mut input).unwrap();
    }

    if input.is_empty() {
        input.push_str(&prompt_customizaton_text.unwrap_or_default());
        prompt_customizaton_text = None;
    }

    debug!("input: {}", input);
    debug!("promt_customization_text: {:?}", prompt_customizaton_text);

    let prompt = customize_prompt(prompt, &args.prompt_params, prompt_customizaton_text);

    debug!("{:?}", prompt);

    match process_input_with_request(prompt, input, &mut output, args.repeat_input) {
        Ok(prompt) => {
            let toml_string =
                toml::to_string(&prompt).expect("Failed to serialize prompt after response.");
            let mut file = fs::File::create(conversation_file_path())
                .expect("Failed to create the conversation save file.");
            file.write_all(toml_string.as_bytes())
                .expect("Failed to write to the conversation file.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ensure_config_usable();
            std::process::exit(1);
        }
    }
}

/// Fills prompt_customization_text with the correct part of the args
/// first arg -> input_or_config_ref
/// second arg -> input_if_config_ref
/// if first arg is a prompt name, get that prompt and use second arg as input
/// if not, use default prompt, use first arg as input and forbid second arg
/// when using voice, only a prompt name can be provided
fn get_default_and_or_custom_prompt(
    args: &Cli,
    prompt_customization_text: &mut Option<String>,
) -> Prompt {
    let mut prompts = get_prompts();
    let available_prompts: Vec<&String> = prompts.keys().collect();
    let prompt_not_found_error = format!(
        "`default` prompt not found, available ones are: {:?}",
        &available_prompts
    );

    let input_or_config_ref = args
        .input_or_template_ref
        .clone()
        .unwrap_or_else(|| String::from("default"));

    if let Some(prompt) = prompts.remove(&input_or_config_ref) {
        if args.input_if_template_ref.is_some() {
            // first arg matching a prompt and second one is customization
            if args.voice {
                panic!(
                    "Invalid parameters, when using voice, either provide a valid ref to a config prompt or nothing at all.\n\
                    Use `sc -v <config_ref>` or `sc -v`"
                );
            }
            *prompt_customization_text = args.input_if_template_ref.clone()
        }
        if args.voice {
            *prompt_customization_text = voice::record_voice_and_get_transcript();
        }
        prompt
    } else {
        *prompt_customization_text = Some(input_or_config_ref);
        if args.input_if_template_ref.is_some() {
            // first arg isn't a prompt and a second one was provided
            panic!(
                "Invalid parameters, either provide a valid ref to a config prompt then an input, or only an input.\n\
                Use `sc <config_ref> \"<your_prompt\"` or `sc \"<your_prompt>\"`"
            );
        } else if args.voice {
            panic!(
                "Invalid parameters, when using voice, either provide a valid ref to a config prompt or nothing at all.\n\
                Use `sc -v <config_ref>` or `sc -v`"
            );
        }

        prompts
            .remove(DEFAULT_PROMPT_NAME)
            .expect(&prompt_not_found_error)
    }
}

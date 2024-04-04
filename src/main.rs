use clap::{Args, Parser};
use log::debug;
use std::fs;
use std::io;
use std::io::{Read, Write};

mod cutsom_prompt;
mod input_processing;
mod request;

#[allow(dead_code)]
mod config;

#[derive(Debug, Parser)]
#[command(
    name = "smartcat (sc)",
    author = "Emilien Fugier",
    version = "0.7.2",
    about = "Putting a brain behind `cat`. CLI interface to bring language models in the Unix ecosystem 🐈‍⬛",
    long_about = None
)]
struct Cli {
    #[arg(default_value_t = String::from("default"))]
    /// which prompt in the config to fetch
    config_prompt: String,
    /// skip reading from stdin and use that value instead
    #[arg(short, long)]
    input: Option<String>,
    /// whether to extend the previous conversation or start a new one
    #[arg(short, long)]
    extend_conversation: bool,
    /// whether to repeat the input before the output, useful to extend instead of replacing
    #[arg(short, long)]
    repeat_input: bool,
    /// skip reading from the input and read this file instead
    #[arg(short, long)]
    file: Option<String>,
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
    /// custom prompt to append before the input
    #[arg(short = 'p', long)]
    custom_prompt: Option<String>,
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
    let args = Cli::parse();

    let mut output = io::stdout();
    let mut input: Box<dyn Read> = match args.file {
        Some(file) => Box::new(
            fs::File::open(&file)
                .unwrap_or_else(|error| panic!("File {} not found. {:?}", file, error)),
        ),
        _ => Box::new(io::stdin()),
    };

    // case for testing
    // TODO: mock API
    if std::env::var("SMARTCAT_TEST").unwrap_or_default() == "1" {
        if let Err(e) = input_processing::chunk_process_input(
            &mut input,
            &mut output,
            "Hello, World!\n```\n",
            "\n```\n",
        ) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    config::ensure_config_files(true)
        .expect("Unable to verify that the config files exist or to generate new ones.");

    let prompt: config::Prompt = if args.extend_conversation {
        let content =
            fs::read_to_string(config::conversation_file_path()).unwrap_or_else(|error| {
                panic!(
                    "Could not read file {:?}, {:?}",
                    config::conversation_file_path(),
                    error
                )
            });
        toml::from_str(&content).expect("failed to load the conversation file")
    } else {
        let mut prompts = config::get_prompts();

        let available_prompts: Vec<&String> = prompts.keys().collect();
        let prompt_not_found_error = format!(
            "Prompt {} not found, availables ones are: {:?}",
            &args.config_prompt, &available_prompts
        );
        prompts
            .remove(&args.config_prompt)
            .expect(&prompt_not_found_error)
    };

    let prompt = cutsom_prompt::customize_prompt(prompt, &args.prompt_params);

    debug!("{:?}", prompt);

    match input_processing::process_input_with_request(
        prompt,
        &mut input,
        args.input,
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

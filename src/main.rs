use clap::{Args, Parser};
use log::debug;
use std::fs;
use std::io;
use std::io::Read;

mod cutsom_prompt;
mod input_processing;
mod request;

#[allow(dead_code)]
mod config;

#[derive(Debug, Parser)]
#[command(
    name = "smartcat (sc)",
    author = "Emilien Fugier",
    version = "0.1",
    about = "Putting a brain behind `cat`. WIP cli interface to language model to bring them in the Unix ecosystem 🐈‍⬛",
    long_about = None
)]
struct Cli {
    /// which prompt in the config to fetch. The config must have at least one named "default"
    /// containing which model and api to hit by default.
    #[arg(default_value_t = String::from("default"))]
    prompt: String,
    #[command(flatten)]
    custom_prompt_args: CustomPrompt,
    /// a system "config" message to send before the first user message
    #[arg(short, long)]
    system_message: Option<String>,
    /// which api to hit
    #[arg(long)]
    api: Option<config::Api>,
    #[arg(short, long)]
    /// which model (of the api) to use
    model: Option<String>,
    /// skip reading from the input and read this file instead
    #[arg(short, long)]
    file: Option<String>,
    /// wether to repeat the input before the output, useful to extend instead of replacing
    #[arg(short, long)]
    repeat_input: bool,
    /// skips reading from input and use that value instead
    #[arg(short, long)]
    input: Option<String>,
}

#[derive(Debug, Args)]
#[group(id = "custom_prompt")]
struct CustomPrompt {
    /// custom prompt to append before the input
    #[arg(short, long)]
    command: Option<String>,
    /// suffix to add after the input and the custom prompt
    #[arg(short, long)]
    after_input: Option<String>,
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

    let mut prompts = config::get_prompts();

    let available_prompts: Vec<&String> = prompts.keys().collect();
    let prompt_not_found_error = format!(
        "Prompt {} not found, availables ones are: {:?}",
        &args.prompt, &available_prompts
    );
    let prompt = prompts.remove(&args.prompt).expect(&prompt_not_found_error);

    let prompt = cutsom_prompt::customize_prompt(
        prompt,
        &args.api,
        &args.model,
        &args.custom_prompt_args.command,
        &args.custom_prompt_args.after_input,
        &args.system_message,
    );

    debug!("{:?}", prompt);

    if let Err(e) = input_processing::process_input_with_request(
        prompt,
        &mut input,
        args.input,
        &mut output,
        args.repeat_input,
    ) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

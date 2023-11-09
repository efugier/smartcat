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
    author = "Emilien Fugier",
    version = "0.1",
    about = "WIP cli interface to language model to bring them in the Unix echosystem",
    long_about = None
)]
struct Cli {
    /// prompt in the config to fetch
    #[arg(group = "prompt_from_config")]
    prompt: Option<String>,
    #[command(flatten)]
    custom_prompt_args: CustomPrompt,
    /// a system "config" message to send before the prompt
    #[arg(short, long)]
    system_message: Option<String>,
    /// which api to hit
    #[arg(long, default_value_t = String::from("openai"))]
    api: String,
    #[arg(short, long, default_value_t = String::from("gpt-3.5-turbo"))]
    /// which model (of the api) to use
    model: String,
    /// file to read input from
    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Debug, Args)]
#[group(id = "custom_prompt", conflicts_with = "prompt_from_config")]
struct CustomPrompt {
    /// custom prompt, incompatible with [PROMTP]
    #[arg(short, long, group = "custom_prompt")]
    command: Option<String>,
    /// prefix to add before custom prompt
    #[arg(short, long, group = "custom_prompt")]
    before: Option<String>,
    /// suffix to add after the imput and the custom prompt
    #[arg(short, long, group = "custom_prompt")]
    after: Option<String>,
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
    if std::env::var("PIPELM_TEST").unwrap_or_default() == "1" {
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

    let mut prompts = config::get_prompts();

    let prompt = match args.prompt {
        Some(prompt) => {
            let available_prompts: Vec<&String> = prompts.keys().collect();
            let prompt_not_found_error = format!(
                "Prompt {} not found, availables ones are: {:?}",
                &prompt, &available_prompts
            );
            prompts.remove(&prompt).expect(&prompt_not_found_error)
        }
        None => config::Prompt {
            api: args.api,
            model: args.model,
            messages: Vec::new(),
        },
    };
    let prompt = cutsom_prompt::customize_prompt(
        prompt,
        &args.custom_prompt_args.command,
        &args.custom_prompt_args.before,
        &args.custom_prompt_args.after,
        &args.system_message,
    );

    debug!("{:?}", prompt);

    if let Err(e) = input_processing::process_input_with_request(prompt, &mut input, &mut output) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

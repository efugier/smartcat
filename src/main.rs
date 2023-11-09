use clap::Parser;
use std::io;
mod input_processing;
mod request;

#[allow(dead_code)]
mod config;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(default_value_t = String::from("default"))]
    prompt: String,
    #[arg(short, long, default_value_t = String::from("openai"))]
    command: String,
}

fn main() {
    let args = Cli::parse();

    let mut output = io::stdout();
    let mut input = io::stdin();

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

    let available_prompts: Vec<&String> = prompts.keys().collect();
    let prompt_not_found_error = format!(
        "Prompt {} not found, availables ones are: {:?}",
        &args.prompt, &available_prompts
    );

    let prompt = prompts
        .get_mut(&args.prompt)
        .expect(&prompt_not_found_error);

    println!("{:?}", prompt);

    if let Err(e) = input_processing::process_input_with_request(prompt, &mut input, &mut output) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

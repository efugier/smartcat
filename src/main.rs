use std::io;
mod config;
mod input_processing;
mod request;

fn main() {
    let mut output = io::stdout();
    let mut input = io::stdin();

    if let Err(e) = input_processing::chunk_process_input(
        &mut input,
        &mut output,
        "Hello, World!\n```\n",
        "\n```\n",
    ) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

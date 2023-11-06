use std::io;
mod input_processing;

fn main() {
    let mut output = io::stdout();
    let mut input = io::stdin();

    if let Err(e) =
        input_processing::process_input(&mut input, &mut output, "Hello, World!\n```\n", "\n```\n")
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

use std::io::{self, Read, Write};

fn main() {
    let mut output = io::stdout();
    let mut input = io::stdin();
    let mut buffer = [0; 1024]; // You can adjust the buffer size as needed.
    let prefix = "Hello wolrd!\n```\n";
    let suffix = "\n```";

    let mut first_chunk = true;

    loop {
        match input.read(&mut buffer) {
            Ok(0) => break, // End of input
            Ok(n) => {
                if first_chunk {
                    if let Err(e) = output.write_all(prefix.as_bytes()) {
                        eprintln!("Error writing to stdout: {}", e);
                        std::process::exit(1);
                    }
                    first_chunk = false;
                }
                if let Err(e) = output.write_all(&buffer[..n]) {
                    eprintln!("Error writing to stdout: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Err(e) = output.write_all(suffix.as_bytes()) {
        eprintln!("Error writing to stdout: {}", e);
        std::process::exit(1);
    }
}

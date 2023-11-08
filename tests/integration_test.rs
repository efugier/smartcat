use std::io::{Read, Write};
use std::process::{Command, Stdio};

#[test]
fn test_io() {
    let hardcoded_prefix = "Hello, World!\n```\n";
    let hardcoded_suffix = "\n```\n";
    let input_data = "Input data";

    // launch the program and get the streams
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("test")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start the program");
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let stdout = child.stdout.as_mut().expect("Failed to open stdout");

    // write
    stdin
        .write_all(input_data.as_bytes())
        .expect("Failed to write to stdin");
    drop(child.stdin.take());

    // read
    let mut output_data = Vec::new();
    stdout
        .read_to_end(&mut output_data)
        .expect("Failed to read from stdout");

    // check
    let status = child.wait().expect("Failed to wait for child process");
    assert!(status.success());
    let expected_output = format!("{}{}{}", hardcoded_prefix, input_data, hardcoded_suffix);
    assert_eq!(String::from_utf8_lossy(&output_data), expected_output);
}

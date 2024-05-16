use std::process::{Child, Command};

pub(super) fn start_recording(recording_command: String) -> Option<Child> {
    // default commands for each os are defined in src/config/voice.rs
    Command::new(recording_command.split_whitespace().next().unwrap())
        .args(recording_command.split_whitespace().skip(1))
        .spawn()
        .ok()
}

pub(super) fn stop_recording(process: &mut Child) {
    process.kill().expect("Failed to stop recording.");
}

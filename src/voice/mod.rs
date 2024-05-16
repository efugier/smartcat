mod api_call;
mod recording;
mod response_schemas;

use device_query::{DeviceQuery, DeviceState, Keycode};
use log::debug;
use std::time::Instant;

use crate::config::api::get_api_config;
use crate::config::prompt::audio_file_path;
use crate::config::voice::{get_voice_config, AUDIO_FILE_PATH_PLACEHOLDER};

use self::api_call::post_audio_and_get_transcript;
use self::recording::{start_recording, stop_recording};

pub fn record_voice_and_get_transcript() -> Option<String> {
    let voice_config = get_voice_config();

    let recording_command = voice_config.recording_command.replace(
        AUDIO_FILE_PATH_PLACEHOLDER,
        audio_file_path()
            .to_str()
            .expect("Unable to parse audio file path to str."),
    );

    let mut process = start_recording(recording_command)?;

    let device_state = DeviceState::new();

    let start_time = Instant::now();
    loop {
        let keys: Vec<Keycode> = device_state.get_keys();
        if keys.contains(&Keycode::Space) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    stop_recording(&mut process);
    debug!("Recording duration: {:?}", start_time.elapsed());

    std::thread::sleep(std::time::Duration::from_millis(250));

    let api_config = get_api_config(&voice_config.api.to_string());

    let transcript = post_audio_and_get_transcript(&api_config.get_api_key())
        .expect("Failed to send audio file to API");

    debug!("Audio transcript: {}", transcript);

    Some(transcript)
}

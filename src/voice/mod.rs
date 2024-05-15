pub mod schemas;

use device_query::{DeviceQuery, DeviceState, Keycode};
use std::process::{Child, Command};

use self::schemas::{OpenAiVoiceResponse, VoiceConfig};
use super::config::{api::get_api_config, prompt::audio_file_path};
use super::third_party::handle_api_response;

fn start_recording() -> Option<Child> {
    let os_string = audio_file_path().into_os_string();
    let audio_file_path = os_string;

    match std::env::consts::OS {
        "windows" => Command::new("cmd")
            .args(["/C", "start", "rec"])
            .arg(audio_file_path)
            .spawn()
            .ok(),
        "macos" => Command::new("sox")
            .arg("-d")
            .arg(audio_file_path)
            .spawn()
            .ok(),
        "linux" => Command::new("arecord")
            .arg("-f")
            .arg("S16_LE")
            .arg("--quiet")
            .arg(audio_file_path)
            .spawn()
            .ok(),
        os => panic!("Unexpected os: {}", os),
    }
}

fn stop_recording(process: &mut Child) {
    process.kill().expect("Failed to stop recording.");
}

pub fn get_voice_transcript() -> Option<String> {
    use std::time::Instant;

    let mut process = start_recording()?;
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
    let duration = start_time.elapsed();
    println!("Recording duration: {:?}", duration);
    std::thread::sleep(std::time::Duration::from_secs(1));
    let voice_config = VoiceConfig::default();

    let api_config = get_api_config(&voice_config.api.to_string());

    let transcript =
        post_audio(&api_config.get_api_key()).expect("Failed to send audio file to API");

    Some(transcript)
}

fn post_audio(api_key: &str) -> reqwest::Result<String> {
    let client = reqwest::blocking::Client::new();
    let form = reqwest::blocking::multipart::Form::new()
        .text("model", "whisper-1")
        .file("file", audio_file_path())
        .expect("Failed to read audio file.");

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()?;

    Ok(handle_api_response::<OpenAiVoiceResponse>(response))
}

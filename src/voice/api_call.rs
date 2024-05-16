use crate::config::prompt::audio_file_path;
use crate::utils::handle_api_response;

use super::response_schemas::OpenAiVoiceResponse;

pub(super) fn post_audio_and_get_transcript(api_key: &str) -> reqwest::Result<String> {
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

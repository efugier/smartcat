use serde::{Deserialize, Serialize};

use crate::config::api::Api;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct VoiceConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub api: Api,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        VoiceConfig {
            url: "https://api.openai.com/v1/audio/transcriptions".to_string(),
            recording_command: None,
            model: Some("whisper-1".to_string()),
            api: Api::Openai,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAiVoiceResponse {
    text: String,
}

impl From<OpenAiVoiceResponse> for String {
    fn from(response: OpenAiVoiceResponse) -> Self {
        response.text
    }
}

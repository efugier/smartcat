use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OpenAiVoiceResponse {
    text: String,
}

impl From<OpenAiVoiceResponse> for String {
    fn from(response: OpenAiVoiceResponse) -> Self {
        response.text
    }
}

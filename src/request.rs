use serde::{Deserialize, Serialize};

use crate::config::ServiceConfig;

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub system_fingerprint: String,
}

pub fn make_authenticated_request(
    service_config: ServiceConfig,
    data: impl Serialize,
) -> Result<ureq::Response, ureq::Error> {
    println!("Trying to reach openai with {}", service_config.api_key);
    ureq::post(&service_config.url)
        .set("Content-Type", "application/json")
        .set(
            "Authorization",
            &format!("Bearer {}", service_config.api_key),
        )
        .send_json(data)
    //     .send_json(ureq::json!(
    //         {
    //     "model": "gpt-4-1106-preview",
    //     "messages": [
    //       {
    //         "role": "system",
    //         "content": "You are a poetic assistant, skilled in explaining complex programming concepts with creative flair."
    //       },
    //       {
    //         "role": "user",
    //         "content": data.messages.last().unwrap().content
    //       }
    //     ]
    //     })
    // )
}

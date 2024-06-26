pub const IS_NONINTERACTIVE_ENV_VAR: &str = "SMARTCAT_NONINTERACTIVE";

/// clean error logging
pub fn handle_api_response<T: serde::de::DeserializeOwned + Into<String>>(
    response: reqwest::blocking::Response,
) -> String {
    let status = response.status();
    if response.status().is_success() {
        response.json::<T>().unwrap().into()
    } else {
        let error_text = response.text().unwrap();
        panic!("API request failed with status {}: {}", status, error_text);
    }
}

pub fn is_interactive() -> bool {
    std::env::var(IS_NONINTERACTIVE_ENV_VAR).unwrap_or_default() != "1"
}

pub fn read_user_input() -> String {
    let mut user_input = String::new();
    std::io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");
    user_input.trim().to_string()
}

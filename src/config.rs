use std::fs;
use toml::Value;

pub fn get_api_key() -> String {
    let config_path = format!(
        "{}/.config/pipelm/.api_configs.toml",
        std::env::var("HOME").unwrap()
    );
    let content = fs::read_to_string(config_path).expect("Failed to read the TOML file");
    let value: Value = content.parse().expect("Failed to parse TOML");

    // Extract the API key from the TOML table.
    let api_key = value
        .get("openai")
        .and_then(|table| table.get("API_KEY"))
        .and_then(|api_key| api_key.as_str())
        .unwrap_or_else(|| {
            eprintln!("API_KEY not found in the TOML file.");
            std::process::exit(1);
        });

    api_key.to_string()
}

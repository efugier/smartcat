use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const VOICE_CONFIG_FILE: &str = "voice.toml";
pub const AUDIO_FILE_PATH_PLACEHOLDER: &str = "<audio_file_path_placeholder>";

use super::{api::Api, resolve_config_path};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VoiceConfig {
    pub url: String,
    pub recording_command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub api: Api,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        let recording_command: String = match std::env::consts::OS {
            "windows" => format!("sox -t waveaudio 0 -d {}", AUDIO_FILE_PATH_PLACEHOLDER),
            "macos" => format!("sox -t waveaudio 0 -d {}", AUDIO_FILE_PATH_PLACEHOLDER),
            "linux" => format!("arecord -f S16_LE --quiet {}", AUDIO_FILE_PATH_PLACEHOLDER),
            os => panic!("Unexpected os: {}", os),
        };

        VoiceConfig {
            url: "https://api.openai.com/v1/audio/transcriptions".to_string(),
            recording_command,
            model: Some("whisper-1".to_string()),
            api: Api::Openai,
        }
    }
}

pub(super) fn voice_config_path() -> PathBuf {
    resolve_config_path().join(VOICE_CONFIG_FILE)
}

pub(super) fn generate_voice_file() -> std::io::Result<()> {
    let voice_config = VoiceConfig::default();

    std::fs::create_dir_all(voice_config_path().parent().unwrap())?;

    let mut voice_config_file = fs::File::create(voice_config_path())?;

    let voice_config_str = toml::to_string_pretty(&voice_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    voice_config_file.write_all(voice_config_str.as_bytes())?;
    Ok(())
}

pub fn get_voice_config() -> VoiceConfig {
    let content = fs::read_to_string(voice_config_path()).unwrap_or_else(|error| {
        panic!("Could not read file {:?}, {:?}", voice_config_path(), error)
    });

    toml::from_str(&content).expect("Unble to parse voice file content into config struct")
}

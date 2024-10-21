pub mod api;
pub mod prompt;
pub mod voice;

use std::{path::PathBuf, process::Command};

use self::{
    api::{api_keys_path, generate_api_keys_file, get_api_config},
    prompt::{generate_prompts_file, get_prompts, prompts_path},
    voice::{generate_voice_file, voice_config_path},
};
use crate::utils::is_interactive;

pub const PLACEHOLDER_TOKEN: &str = "#[<input>]";

const DEFAULT_CONFIG_PATH: &str = ".config/smartcat/";
const CUSTOM_CONFIG_ENV_VAR: &str = "SMARTCAT_CONFIG_PATH";

fn resolve_config_path() -> PathBuf {
    if let Ok(custom_path) = std::env::var(CUSTOM_CONFIG_ENV_VAR) {
        PathBuf::from(custom_path)
    } else {
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE")
        } else {
            std::env::var("HOME")
        };

        match home_dir {
            Ok(dir) => PathBuf::from(dir).join(DEFAULT_CONFIG_PATH),
            Err(_) => panic!(
                "Could not determine default config path. Set either ${CUSTOM_CONFIG_ENV_VAR} or {} environment variable",
                if cfg!(windows) { "%USERPROFILE%" } else { "$HOME" }
            ),
        }
    }
}

pub fn ensure_config_files() -> std::io::Result<()> {
    let interactive = is_interactive();

    if !prompts_path().exists() {
        if interactive {
            println!(
                "Prompt config file not found at {:?}, generating one.\n...",
                prompts_path()
            );
        }
        generate_prompts_file()?
    }

    if !voice_config_path().exists() {
        println!(
            "Voice config file not found at {:?}, generating one.\n...",
            ()
        );
        generate_voice_file().expect("Unable to generate config files");
    };

    if !api_keys_path().exists() {
        println!(
            "API config file not found at {:?}, generating one.\n...",
            api_keys_path()
        );
        generate_api_keys_file().expect("Unable to generate config files");
        if interactive {
            ensure_config_usable();
        }
    };

    Ok(())
}

pub fn ensure_config_usable() {
    let interactive = is_interactive();

    // check if any config has an API key;
    let third_parth_config_usable = get_prompts().iter().any(|(_, prompt)| {
        let api = get_api_config(&prompt.api.to_string());
        api.api_key.is_some() || api.api_key_command.is_some()
    });
    if !third_parth_config_usable {
        println!(
            "No API key is configured.\n\
            How to configure your API keys:\n\
            https://github.com/efugier/smartcat/#configuration\n"
        );
    }

    // check if local execution is possible with Ollama
    if !is_executable_in_path("ollama") {
        println!(
            "Ollama not found in PATH.\n\
            How to setup Ollama:\n\
            https://github.com/efugier/smartcat#ollama-setup"
        );
    }

    // nothing is setup
    if interactive && !third_parth_config_usable && !is_executable_in_path("ollama") {
        println!("\nInstall Ollama or set an api key for at least one of the providers to get started, then come back!");
        std::process::exit(1);
    }
}

fn is_executable_in_path(executable_name: &str) -> bool {
    Command::new("which")
        .arg(executable_name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{
            api::{api_keys_path, default_timeout_seconds, Api, ApiConfig},
            ensure_config_files,
            prompt::{prompts_path, Prompt},
            resolve_config_path,
            voice::{voice_config_path, VoiceConfig},
            CUSTOM_CONFIG_ENV_VAR, DEFAULT_CONFIG_PATH,
        },
        utils::IS_NONINTERACTIVE_ENV_VAR,
    };
    use serial_test::serial;
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::io::Read;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    #[serial]
    fn resolver_custom_config_path() {
        let temp_path = "/tmp/custom_path";
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);

        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_path);
        let result = resolve_config_path();

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        assert_eq!(result, Path::new(temp_path));
    }

    #[test]
    #[serial]
    fn resolve_default_config_path() {
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);

        env::remove_var(CUSTOM_CONFIG_ENV_VAR);
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE")
        } else {
            std::env::var("HOME")
        }
        .expect("HOME not defined");

        let default_path = PathBuf::new().join(home_dir).join(DEFAULT_CONFIG_PATH);
        let result = resolve_config_path();

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        assert_eq!(result, Path::new(&default_path));
    }

    #[test]
    #[serial]
    fn test_ensure_config_files_not_existing() -> std::io::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);
        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_dir.path());
        env::set_var(IS_NONINTERACTIVE_ENV_VAR, "1");

        let api_keys_path = api_keys_path();
        let prompts_path = prompts_path();
        let voice_path = voice_config_path();

        assert!(!api_keys_path.exists());
        assert!(!prompts_path.exists());
        assert!(!voice_path.exists());

        let result = ensure_config_files();

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        result?;

        assert!(api_keys_path.exists());
        assert!(prompts_path.exists());
        assert!(voice_path.exists());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_ensure_config_files_already_existing() -> std::io::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;

        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);
        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_dir.path());
        env::set_var(IS_NONINTERACTIVE_ENV_VAR, "1");

        let api_keys_path = api_keys_path();
        let prompts_path = prompts_path();
        let voice_path = voice_config_path();

        // Precreate files with some content
        let mut api_keys_file = fs::File::create(&api_keys_path)?;
        api_keys_file.write_all(b"Some API key data")?;

        let mut prompts_file = fs::File::create(&prompts_path)?;
        prompts_file.write_all(b"Some prompts data")?;

        let mut voice_file = fs::File::create(&voice_path)?;
        voice_file.write_all(b"Some voice data")?;

        let result = ensure_config_files();

        // Restoring the original environment variable
        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        result?;

        // Check if files still exist
        assert!(api_keys_path.exists());
        assert!(prompts_path.exists());
        assert!(voice_path.exists());

        // Check if the contents remain unchanged
        let mut api_keys_content = String::new();
        fs::File::open(&api_keys_path)?.read_to_string(&mut api_keys_content)?;
        assert_eq!(api_keys_content, "Some API key data".to_string());

        let mut prompts_content = String::new();
        fs::File::open(&prompts_path)?.read_to_string(&mut prompts_content)?;
        assert_eq!(prompts_content, "Some prompts data".to_string());

        let mut voice_content = String::new();
        fs::File::open(&voice_path)?.read_to_string(&mut voice_content)?;
        assert_eq!(voice_content, "Some voice data".to_string());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_ensure_config_files_serialization() -> std::io::Result<()> {
        // Setup paths
        let temp_dir = tempfile::TempDir::new()?;
        let original_value = env::var(CUSTOM_CONFIG_ENV_VAR);
        env::set_var(CUSTOM_CONFIG_ENV_VAR, temp_dir.path());
        env::set_var(IS_NONINTERACTIVE_ENV_VAR, "1");

        let api_keys_path = api_keys_path();
        let prompts_path = prompts_path();
        let voice_path = voice_config_path();

        assert!(!api_keys_path.exists());
        assert!(!prompts_path.exists());
        assert!(!voice_path.exists());

        let result = ensure_config_files();

        match original_value {
            Ok(val) => env::set_var(CUSTOM_CONFIG_ENV_VAR, val),
            Err(_) => env::remove_var(CUSTOM_CONFIG_ENV_VAR),
        }

        result?;

        // Read back the files and deserialize
        let api_config_contents = fs::read_to_string(&api_keys_path)?;
        let prompts_config_contents = fs::read_to_string(&prompts_path)?;
        let voice_file_content = fs::read_to_string(&voice_path)?;

        // Deserialize contents to expected data structures
        // TODO: would be better to use `get_config` and `get_prompts` but
        // current implementation does not allow for error management that would
        // enable safe environement variable manipulation
        let api_config: HashMap<String, ApiConfig> =
            toml::from_str(&api_config_contents).expect("Failed to deserialize API config");

        let prompt_config: HashMap<String, Prompt> =
            toml::from_str(&prompts_config_contents).expect("Failed to deserialize prompts config");

        let voice_config: VoiceConfig =
            toml::from_str(&voice_file_content).expect("Failed to deserialize voice config");

        // Check if the content matches the default values

        // API
        for (api, expected_config) in [
            (Prompt::default().api.to_string(), ApiConfig::default()),
            (Api::Mistral.to_string(), ApiConfig::mistral()),
            (Api::Groq.to_string(), ApiConfig::groq()),
            (Api::Anthropic.to_string(), ApiConfig::anthropic()),
        ] {
            let config = api_config.get(&api).unwrap();
            assert_eq!(
                ApiConfig {
                    timeout_seconds: default_timeout_seconds(),
                    ..expected_config
                },
                *config
            );
        }

        // Prompts
        let default_prompt = Prompt::default();
        assert_eq!(prompt_config.get("default"), Some(&default_prompt));

        let empty_prompt = Prompt::empty();
        assert_eq!(prompt_config.get("empty"), Some(&empty_prompt));

        // Voice
        assert_eq!(voice_config, VoiceConfig::default());

        Ok(())
    }
}

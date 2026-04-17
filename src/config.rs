use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HelpLanguage {
    Ja,
    #[default]
    En,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub help: HelpConfig,
    #[serde(default)]
    pub copy: CopyConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct HelpConfig {
    #[serde(default)]
    pub language: HelpLanguage,
}

#[derive(Debug, Default, Deserialize)]
pub struct CopyConfig {
    #[serde(default, deserialize_with = "deserialize_optional_path")]
    pub after_copy_hook: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let Some(config_path) = Self::config_path() else {
            return Config::default();
        };

        if let Ok(content) = fs::read_to_string(&config_path) {
            match toml::from_str(&content) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("Failed to parse config {}: {}", config_path.display(), err);
                    Config::default()
                }
            }
        } else {
            Config::default()
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
            Some(
                PathBuf::from(config_home)
                    .join("pocoshelf")
                    .join("config.toml"),
            )
        } else {
            directories::BaseDirs::new().map(|dirs| {
                dirs.home_dir()
                    .join(".config")
                    .join("pocoshelf")
                    .join("config.toml")
            })
        }
    }
}

fn deserialize_optional_path<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::{Config, HelpLanguage};

    #[test]
    fn help_language_defaults_to_english() {
        let config = Config::default();

        assert_eq!(config.help.language, HelpLanguage::En);
    }

    #[test]
    fn help_language_can_be_loaded_from_config() {
        let config: Config = toml::from_str(
            r#"
[help]
language = "ja"
"#,
        )
        .expect("config should parse");

        assert_eq!(config.help.language, HelpLanguage::Ja);
    }

    #[test]
    fn after_copy_hook_defaults_to_none() {
        let config = Config::default();

        assert_eq!(config.copy.after_copy_hook, None);
    }

    #[test]
    fn after_copy_hook_can_be_loaded_from_config() {
        let config: Config = toml::from_str(
            r#"
[copy]
after_copy_hook = "/Users/rc/bin/after-copy"
"#,
        )
        .expect("config should parse");

        assert_eq!(
            config.copy.after_copy_hook.as_deref(),
            Some(std::path::Path::new("/Users/rc/bin/after-copy"))
        );
    }

    #[test]
    fn empty_after_copy_hook_is_ignored() {
        let config: Config = toml::from_str(
            r#"
[copy]
after_copy_hook = ""
"#,
        )
        .expect("config should parse");

        assert_eq!(config.copy.after_copy_hook, None);
    }
}

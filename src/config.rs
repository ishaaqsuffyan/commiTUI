use serde::{Deserialize}; // No need for Deserializer or HashMap directly for this merge approach
use std::{fs, path::PathBuf};

// Add `dirs = "5"` to your Cargo.toml if you haven't already.
use dirs;

// --- Config Struct and Default Values (Same as before) ---
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_types")]
    pub types: Vec<String>,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,

    pub subject_max_length: usize,
    pub subject_start_lowercase: bool,
    pub subject_no_ending_period: bool,
}

fn default_types() -> Vec<String> {
    vec![
        "feat".into(), "fix".into(), "docs".into(), "style".into(), "refactor".into(),
        "perf".into(), "test".into(), "build".into(), "ci".into(), "chore".into(), "revert".into()
    ]
}

fn default_scopes() -> Vec<String> {
    vec![
        "no scope".into(),
        "core".into(), "api".into(), "ui".into(), "auth".into(), "db".into(),
        "test".into(), "build".into(), "deps".into(), "ci".into(),
        "────────────".into(),
        "config".into(), "infra".into(), "release".into(), "chore".into(), "perf".into(),
        "style".into(), "lint".into(), "i18n".into(), "analytics".into(), "security".into(),
        "logging".into(), "devops".into(), "deploy".into(), "assets".into(), "mock".into(), "example".into()
    ]
}

fn default_subject_max_length() -> usize { 72 }
fn default_subject_start_lowercase() -> bool { true }
fn default_subject_no_ending_period() -> bool { true }

// --- Merge Trait (Same as before) ---
pub trait MergeConfig {
    fn merge(&mut self, other: Self);
}

impl MergeConfig for Config {
    fn merge(&mut self, other: Self) {
        // Only override if the 'other' config actually specifies the field
        // This requires making fields optional in Config, which adds complexity for defaults.
        // For simplicity with `#[serde(default)]`, we'll just overwrite.
        // If 'other' is deserialized from a file where a field is missing, `serde(default)`
        // will fill it with the default, effectively overwriting 'self' with that default.
        // This behavior is usually acceptable for overriding.
        
        // If you want more granular merging (e.g., if `other.types` is empty, keep `self.types`),
        // you'd need to change Config fields to Option<Vec<String>>, etc.
        // For now, simpler direct assignment with #[serde(default)] is used.

        self.types = other.types;
        self.scopes = other.scopes;
        self.subject_max_length = other.subject_max_length;
        self.subject_start_lowercase = other.subject_start_lowercase;
        self.subject_no_ending_period = other.subject_no_ending_period;
    }
}

// --- Default Impl for Config (Same as before) ---
impl Default for Config {
    fn default() -> Self {
        Self {
            types: default_types(),
            scopes: default_scopes(),
            subject_max_length: default_subject_max_length(),
            subject_start_lowercase: default_subject_start_lowercase(),
            subject_no_ending_period: default_subject_no_ending_period(),
        }
    }
}

// --- Updated Config::load() and get_global_config_path() ---
impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut final_config = Config::default();

        // 1. Try to load global config (OS-specific path)
        if let Some(global_config_path) = Config::get_global_config_path() {
            // Ensure the parent directory exists before trying to read
            if global_config_path.exists() {
                if let Ok(content) = fs::read_to_string(&global_config_path) {
                    match toml::from_str::<Config>(&content) {
                        Ok(global_config) => {
                            final_config.merge(global_config);
                        },
                        Err(e) => eprintln!("Warning: Could not parse global config at {}: {}", global_config_path.display(), e),
                    }
                } else {
                    // This branch would only be hit if path.exists() was true but read_to_string failed for other reasons
                    eprintln!("Warning: Could not read global config at {}", global_config_path.display());
                }
            }
        }

        // 2. Try to load local config (./commitui.toml)
        let local_config_paths = ["./commitui.toml"];
        for path in &local_config_paths {
            if let Ok(content) = fs::read_to_string(path) {
                match toml::from_str::<Config>(&content) {
                    Ok(local_config) => {
                        final_config.merge(local_config); // Local overrides global
                        // If a local config is found and successfully parsed, it's the final source.
                        // We return here to prevent further fallback.
                        return Ok(final_config);
                    },
                    Err(e) => eprintln!("Warning: Could not parse local config at {}: {}", path, e),
                }
            }
        }

        // If no local config found or parse error, return the merged global/default config
        Ok(final_config)
    }

    fn get_global_config_path() -> Option<PathBuf> {
        if let Some(mut config_dir) = dirs::config_dir() {
            // Append your application's name and config file name
            config_dir.push("commiTUI");
            config_dir.push("config.toml");
            Some(config_dir)
        } else {
            None
        }
    }
}
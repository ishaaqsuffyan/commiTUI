use serde::{Deserialize};
use std::{fs, path::PathBuf};
use dirs; // For platform-specific config directory

// Config Struct
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // Commit Types
    pub types: Option<Vec<String>>,

    // Scopes
    pub scopes: Option<Vec<String>>,

    // Subject Validation Rules
    pub subject_max_length: Option<usize>,
    pub subject_start_lowercase: Option<bool>,
    pub subject_no_ending_period: Option<bool>,

    // Add more configurable validation rules here as needed (as Option<Type>)
}

// --- Default Values for Config Fields (these are the true defaults) ---
// MAKE THESE PUBLIC!
pub fn default_types() -> Vec<String> { // <--- ADD pub
    vec![
        "feat".into(), "fix".into(), "docs".into(), "style".into(), "refactor".into(),
        "perf".into(), "test".into(), "build".into(), "ci".into(), "chore".into(), "revert".into()
    ]
}

pub fn default_scopes() -> Vec<String> { // <--- ADD pub
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

pub fn default_subject_max_length() -> usize { 72 } // <--- ADD pub
pub fn default_subject_start_lowercase() -> bool { true } // <--- ADD pub
pub fn default_subject_no_ending_period() -> bool { true } // <--- ADD pub


// --- Trait for Merging Configs ---
pub trait MergeConfig {
    fn merge(&mut self, other: Self);
}

impl MergeConfig for Config {
    fn merge(&mut self, other: Self) {
        if let Some(types) = other.types {
            self.types = Some(types);
        }
        if let Some(scopes) = other.scopes {
            self.scopes = Some(scopes);
        }
        if let Some(length) = other.subject_max_length {
            self.subject_max_length = Some(length);
        }
        if let Some(lowercase) = other.subject_start_lowercase {
            self.subject_start_lowercase = Some(lowercase);
        }
        if let Some(no_period) = other.subject_no_ending_period {
            self.subject_no_ending_period = Some(no_period);
        }
    }
}


// --- Default Implementation for Config (used if no files found/first base config) ---
impl Default for Config {
    fn default() -> Self {
        Self {
            types: Some(default_types()),
            scopes: Some(default_scopes()),
            subject_max_length: Some(default_subject_max_length()),
            subject_start_lowercase: Some(default_subject_start_lowercase()),
            subject_no_ending_period: Some(default_subject_no_ending_period()),
        }
    }
}

// --- Config Loading Logic ---
impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut final_config = Config::default();

        // 1. Try to load global config
        if let Some(global_config_path) = Config::get_global_config_path() {
            if global_config_path.exists() {
                if let Ok(content) = fs::read_to_string(&global_config_path) {
                    match toml::from_str::<Config>(&content) {
                        Ok(global_config) => {
                            final_config.merge(global_config);
                        },
                        Err(e) => eprintln!("Warning: Could not parse global config at {}: {}", global_config_path.display(), e),
                    }
                } else {
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
                        final_config.merge(local_config);
                        return Ok(final_config);
                    },
                    Err(e) => eprintln!("Warning: Could not parse local config at {}: {}", path, e),
                }
            }
        }

        Ok(final_config)
    }

    pub fn get_global_config_path() -> Option<PathBuf> {
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("commiTUI");
            config_dir.push("config.toml");
            Some(config_dir)
        } else {
            None
        }
    }
}
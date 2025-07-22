use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            types: vec![
                "feat".into(), "fix".into(), "docs".into(), "style".into(), "refactor".into(),
                "perf".into(), "test".into(), "build".into(), "ci".into(), "chore".into(), "revert".into()
            ],
            scopes: vec![
                "no scope".into(),
                "core".into(), "api".into(), "ui".into(), "auth".into(), "db".into(),
                "test".into(), "build".into(), "deps".into(), "ci".into(),
                "────────────".into(),
                "config".into(), "infra".into(), "release".into(), "chore".into(), "perf".into(),
                "style".into(), "lint".into(), "i18n".into(), "analytics".into(), "security".into(),
                "logging".into(), "devops".into(), "deploy".into(), "assets".into(), "mock".into(), "example".into()
            ],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let paths = [".commituirc", "commitui.toml"];
        for path in &paths {
            if let Ok(content) = fs::read_to_string(path) {
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }
        Err("No config file found, using defaults.".into())
    }
}
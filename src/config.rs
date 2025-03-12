use std::env;
use std::fs::File;
pub struct ProjectConfig {
    pub name: String,
    pub path: String,
    pub command: String,
}

pub struct Config {
    pub projects: Vec<ProjectConfig>,
}

impl Config {
    pub fn new() -> Self {
        let config_path = env::var("WORK_PROJECT_CONFIG").unwrap_or_else(|_| "config.json".to_string());
        let config_file = File::open(config_path);
    }
}
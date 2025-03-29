use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfig {
    // 项目名称
    pub name: String,
    // 项目所在的文件系统路径
    pub path: String,
    // 启动项目的命令，通常是 node 命令
    pub start_command: String,
    // 最小内存使用量，单位为 MB
    pub min_memory_usage: u32,
    // 指定的node版本
    pub node_version: Option<String>,
}

#[allow(dead_code)]
pub struct Config {
    pub projects: Vec<ProjectConfig>,
}

pub struct ReadConfig;

impl ReadConfig {
    pub fn new() -> Vec<ProjectConfig> {
        let config_path = get_config_path("WORK_PROJECT_CONFIG");
        println!("Loading config: {}", config_path);

        let file = get_json_file(&config_path);

        let reader = BufReader::new(file);

        match serde_json::from_reader::<BufReader<File>, Vec<ProjectConfig>>(reader) {
            Ok(json) => json,
            Err(e) => {
                panic!("Failed to parse config file: {}", e);
            }
        }
    }
}

fn get_config_path(key: &str) -> String {
    match env::var(key) {
        Ok(path) => path,
        Err(_) => {
            panic!("Environment variable {} not set", key);
        }
    }
}

fn get_json_file(path: &String) -> File {
    match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Failed to open config file: {}", path);
        }
    }
}
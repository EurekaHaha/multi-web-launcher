use std::fmt::Debug;
use std::io::Result;
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;
use sysinfo::System;

use crate::config::ProjectConfig;

#[derive(Debug)]
pub struct Project {
    pub project_info: ProjectConfig,
    // 当前运行的进程实例，如果项目未运行则为 None
    pub process: Option<Child>,
    // 进程 ID，用于跟踪和管理进程
    pid: Option<u32>,
    // 项目当前的运行状态
    pub status: Status,
    // 项目最后一次启动的时间
    last_run_time: Option<std::time::SystemTime>,
    // 是否在进程意外终止时自动重启
    auto_restart: bool,
}

#[derive(Debug)]
pub enum Status {
    Running,
    Stopped,
    Failed,
    Unknown,
}

pub enum CommandType {
    Default(String),
    Specific(String),
}

impl Project {
    pub fn new(project_info: ProjectConfig) -> Self {
        Self {
            project_info: project_info,
            process: None,
            pid: None,
            status: Status::Stopped,
            last_run_time: None,
            auto_restart: false,
        }
    }
}

impl Project {
    fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn set_process(&mut self, process: Option<Child>) {
        self.process = process;
    }

    pub fn set_pid(&mut self, pid: u32) {
        self.pid = if pid == 0 { None } else { Some(pid) };
    }

    pub fn set_last_run_time(&mut self, last_run_time: SystemTime) {
        self.last_run_time = Some(last_run_time);
    }
}

impl Project {
    pub fn start(&mut self, sys: &mut System) -> Result<()> {
        // 如果有进程在运行 先停止
        if let Some(mut p) = self.process.take() {
            p.kill().unwrap();
        }

        // 解析命令字符串
        let mut parts = self.project_info.start_command.split_whitespace();
        let command_str = parts.next().unwrap_or("npm").to_string();
        // * collect会将一个迭代器转化为标注的类型 此处标记的是Vec<&str>
        let args: Vec<&str> = parts.collect();

        sys.refresh_memory();

        println!("当前空闲内存: {} MB", sys.free_memory() / 1024 / 1024);

        println!("当前项目: {:?}", self.project_info.node_version);

        let mut command = get_command_by_config(
            &self.project_info.node_version,
            &self.project_info.min_memory_usage,
            command_str,
        );

        println!("当前Node版本: {}", get_node_version());

        match command
            .args(&args)
            .current_dir(&self.project_info.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                self.set_pid(child.id());
                self.set_process(Some(child));
                self.set_status(Status::Running);
                self.set_last_run_time(SystemTime::now());

                Ok(())
            }
            Err(e) => {
                self.set_status(Status::Failed);
                Err(e)
            }
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            match process.kill() {
                Ok(_) => {
                    println!("[{}]已停止", self.project_info.name);
                }
                Err(e) => {
                    self.process = Some(process);
                    return Err(e);
                }
            }
        }

        self.set_status(Status::Stopped);
        self.set_pid(0);
        self.set_process(None);

        Ok(())
    }
}

fn get_command_by_config(
    version: &Option<String>,
    memories: &Option<u32>,
    command_str: String,
) -> Command {
    println!("version: {:?}", version);
    println!("command_str: {:?}", command_str);
    let program_command: CommandType = match version {
        Some(v) => get_npm_version_command(&v),
        None => CommandType::Default(command_str),
    };

    // * command::env方法返回的是command的可变引用 如果需要command本身直接返回command即可
    let mut ret_command = match program_command {
        CommandType::Default(cmd) => Command::new(cmd),
        CommandType::Specific(node_path) => {
            let path: String = get_env_by_key("PATH").expect("PATH environment variable not set");
            println!("PATH: {}", path);
            println!("node_path: {}", node_path);
            let mut cmd_obj = Command::new("npm");
            cmd_obj.env("PATH", format!("{node_path}:{path}"));
            cmd_obj
        }
    };

    if let Some(ref memories) = memories {
        ret_command.env("NODE_OPTIONS", format!("--max_old_space_size={}", memories));
    }
    ret_command
}

// 获取指定版本node的路径
fn get_specific_version_node_path(version: &String) -> String {
    let env = get_env_by_key("HOME").expect("HOME environment variable not set");

    format!("{env}/.nvm/versions/node/v{}/bin", version)
}

fn get_node_version() -> String {
    let output = Command::new("node")
        .arg("-v")
        .output()
        .expect("Failed to execute command");

    String::from_utf8(output.stdout).unwrap()
}

fn get_npm_version_command(version: &String) -> CommandType {
    CommandType::Specific(get_specific_version_node_path(version))
}

fn get_env_by_key(key: &str) -> Option<String> {
    if let Ok(env) = std::env::var(key) {
        Some(env)
    } else {
        None
    }
}

// ************************ 暂时不使用 **************************

#[allow(unused)]
fn get_node_version_command(version: &String) -> String {
    let env: String = get_env_by_key("HOME").expect("HOME environment variable not set");
    // let version = match version {
    //     Some(v) => v,
    //     None => String::from("10.16.1"),
    // };
    // * 还可以这么写
    // let version = version.unwrap_or_else(|| String::from("10.16.1"));
    format!("{env}/.nvm/versions/node/v{}/bin/npm", version)
}

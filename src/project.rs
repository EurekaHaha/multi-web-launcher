
use std::fmt::Debug;
use std::process::{ Child, Command, Stdio };
use std::time::SystemTime;
use std::io::Result;
use sysinfo::System;

use crate::config::ProjectConfig;

#[derive(Debug)]
pub struct Project {
    // 项目名称
    pub name: String,
    // 项目所在的文件系统路径
    path: String,
    // 启动项目的命令，通常是 node 命令
    start_command: String,
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
    pub min_memory_usage: u32,
}

#[derive(Debug)]
pub enum Status {
    Running,
    Stopped,
    Failed,
    Unknown,
}

impl Project {
    pub fn new(project_config: ProjectConfig) -> Self {
        Self {
            name: project_config.name,
            path: project_config.path,
            start_command: project_config.start_command,
            min_memory_usage: project_config.min_memory_usage,
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
        let mut parts = self.start_command.split_whitespace();
        let command: &str = parts.next().unwrap_or("npm");
        // * collect会将一个迭代器转化为标注的类型 此处标记的是Vec<&str>
        let args: Vec<&str> = parts.collect();

        sys.refresh_memory();
        let free_memory_mb = sys.free_memory() / 1024 / 1024;
        let node_limit_memory = self.min_memory_usage;
        println!("当前空闲内存: {} MB", free_memory_mb);
        println!("当前命令: {} {:?}", command, args);

        let node_version = get_node_version();
        println!("当前Node版本: {}", node_version);
        change_node_version(Some(node_version));
        let node_version = get_node_version();
       println!("当前Node版本: {}", node_version);
        // 执行命令
        match
            Command::new(command)
                .args(&args)
                .current_dir(&self.path)
                // .env("NODE_OPTIONS", format!("--max_old_space_size={}", node_limit_memory))
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
                    println!("[{}]已停止", self.name);
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

fn get_node_version() -> String {
    let output = Command::new("node")
        .arg("-v")
        .output()
        .expect("Failed to execute command");

    String::from_utf8(output.stdout).unwrap()
}

fn change_node_version(version: Option<String>) {
    let version = match version {
        Some(v) => v,
        None => String::from("10.16.1"),
    };
    // * 还可以这么写
    // let version = version.unwrap_or_else(|| String::from("10.16.1"));
    let nvm_path = "$HOME/.nvm/nvm.sh";
    let command_str = format!("source {} && nvm use {}", nvm_path, version);

    Command::new("bash")
        .arg("-c")
        .arg(command_str)
        .output()
        .expect("Failed to execute command");
}
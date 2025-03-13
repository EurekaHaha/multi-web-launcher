#![allow(unused)]

use std::fmt::{ Debug, Formatter, Error };
use std::process::{ Child, Command, Stdio };
use std::time::SystemTime;
use std::io::Result;
use std::io::{ BufRead, BufReader };
use std::thread;
use crate::config::ProjectConfig;

#[derive(Debug)]
pub struct Project {
    // 项目名称
    name: String,
    // 项目所在的文件系统路径
    path: String,
    // 启动项目的命令，通常是 node 命令
    start_command: String,
    // 当前运行的进程实例，如果项目未运行则为 None
    process: Option<Child>,
    // 进程 ID，用于跟踪和管理进程
    pid: Option<u32>,
    // 项目当前的运行状态
    status: ProjectStatus,
    // 项目最后一次启动的时间
    last_run_time: Option<std::time::SystemTime>,
    // 是否在进程意外终止时自动重启
    auto_restart: bool,
}

pub enum Commands {
    Start,
    Stop,
    Restart,
}

#[derive(Debug)]
pub enum ProjectStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

impl Project {
    pub fn new(project_config: &ProjectConfig) -> Self {
        Self {
            name: project_config.name.clone(),
            path: project_config.path.clone(),
            start_command: project_config.start_command.clone(),
            process: None,
            pid: None,
            status: ProjectStatus::Stopped,
            last_run_time: None,
            auto_restart: false,
        }
    }
}

impl Project {
    fn set_status(&mut self, status: ProjectStatus) {
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
    pub fn start(&mut self) -> Result<()> {
        // 如果有进程在运行 先停止
        if let Some(mut p) = self.process.take() {
            p.kill();
        }

        // 解析命令字符串
        let mut parts = self.start_command.split_whitespace();
        let command = parts.next().unwrap_or("npm");
        let args: Vec<&str> = parts.collect();

        // 执行命令
        match
            Command::new(command)
                .args(&args)
                .current_dir(&self.path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        {
            Ok(mut child) => {
                let stdout = child.stdout.take().unwrap();

                let project_name = self.name.clone();

                // todo 之后这里要移动到其他地方 比如 日志系统
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            println!("[{}]输出: {}", project_name, line);
                        }
                    }
                });

                self.set_pid(child.id());
                self.set_process(Some(child));
                self.set_status(ProjectStatus::Running);
                self.set_last_run_time(SystemTime::now());

                Ok(())
            }
            Err(e) => {
                self.set_status(ProjectStatus::Failed);
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

        self.set_status(ProjectStatus::Stopped);
        self.set_pid(0);
        self.set_process(None);

        Ok(())
    }
}

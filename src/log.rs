use crate::project::{ Project, Status };
use std::io::{ BufReader, BufRead, Result, Error };
use std::collections::VecDeque;
use std::thread;

#[allow(unused)]
pub struct NodeLog {
    pub project: Project,
    pub log_list: VecDeque<String>,
    pub status: Status,
    max_logs: usize,
}

impl NodeLog {
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

impl NodeLog {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            log_list: VecDeque::with_capacity(2000),
            status: Status::Unknown,
            max_logs: 2000,
        }
    }

    pub fn log_start(&mut self) -> Result<()> {
        let process: &mut Option<std::process::Child> = &mut self.project.process;

        println!("process: {:?}", process);

        match process {
            // * rust和js不同 => 不是代表一个函数 是一个模式匹配指后面的表达式的值作为match的返回值 所以return是直接给整个函数返回的
            Some(ref mut process) => {
                // @ 每个流只能take一次 take之后流的所有权就转移了
                let stdout_reader = if let Some(stdout) = process.stdout.take() {
                    BufReader::new(stdout)
                } else {
                    return self.handle_error("stdout is None Log start failed")
                };


                let stderr_reader = if let Some(stderr) = process.stderr.take() {
                    BufReader::new(stderr)
                } else {
                    return self.handle_error("stderr is None Log start failed")
                };

                let project_name_stdout = self.project.project_info.name.clone();
                let project_name_stderr = self.project.project_info.name.clone();
                self.set_status(Status::Running);

                thread::spawn(move || {
                    for line in stdout_reader.lines() {
                        if let Ok(line) = line {
                            // todo 这里之后不会直接打印到控制台 而是打印到一个新的UI中
                            println!("[{}]输出: {}", project_name_stdout, line);
                        }
                    }
                });

                thread::spawn(move || {
                    for line in stderr_reader.lines() {
                        if let Ok(line) = line {
                            // todo 这里之后不会直接打印到控制台 而是打印到一个新的UI中
                            println!("[{}]错误: {}", project_name_stderr, line);
                        }
                    }
                });
                Result::Ok(())
            },
            None => {
                self.handle_error("Process is None")
            }
        }
    }

    fn handle_error(&mut self, error: &str) -> Result<()> {
        self.set_status(Status::Failed);
        self.project.stop().unwrap();
        Result::Err(Error::new(
            std::io::ErrorKind::Other,
            format!("Log start failed: {}", error),
        ))
    }
}

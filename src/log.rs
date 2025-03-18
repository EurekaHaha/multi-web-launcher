use crate::project::{ Project, Status };
use std::io::{ BufReader, BufRead };
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

    pub fn log_start(&mut self) {
        let process: &mut Option<std::process::Child> = &mut self.project.process;

        println!("process: {:?}", process);

        match process {
            Some(ref mut process) => {
                // @ 每个流只能take一次 take之后流的所有权就转移了
                let reader = if let Some(stdout) = process.stdout.take() {
                    BufReader::new(stdout)
                } else {
                    self.project.stop().unwrap();
                    panic!("stdout is None");
                };

                let project_name = self.project.name.clone();
                self.set_status(Status::Running);

                thread::spawn(move || {
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            // todo 这里之后不会直接打印到控制台 而是打印到一个新的UI中
                            println!("[{}]输出: {}", project_name, line);
                        }
                    }
                });
            },
            None => {
                println!("process is None");
            }
        }
    }
}

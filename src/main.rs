mod project;
mod config;
mod log;

use std::collections::HashMap;
use std::io::*;

fn main() {
    // * 测试启动

    let project_config = config::ReadConfig::new();

    println!("project_config: {:?}", project_config);

    let mut project_hashmap: HashMap<_, _> = project_config.into_iter().map(|project: config::ProjectConfig| (project.name, project::Project::new(&project))).collect();

    // let mut proxy = project::Project::new(&project_config[0]);
    let mut logs: HashMap<String, log::NodeLog<'_>> = HashMap::with_capacity(project_hashmap.len());

    for (name, mut project) in project_hashmap {
        println!("正在启动 {}...", name);
        project.start().expect("启动项目 {name} 失败");
        println!("{} 启动成功", name);

        // todo: 此处所有权有问题 需要修改
        let mut log = log::NodeLog::new(&mut project);
        log.log_start();
        logs.insert(name, log);
    }

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            if l.trim() == "exit" {
                
            }
        }
    }

    println!("程序已退出");

}
mod project;
mod config;
mod log;

use std::collections::HashMap;
use std::io::*;
use config::ProjectConfig;
use sysinfo::System;


fn main() {

    let project_config = config::ReadConfig::new();
    println!("project_config: {:?}", project_config);

    let mut sys = System::new();
    print_sys_info(&mut sys);
    // 启动所有项目并监控日志
    let mut logs = start_projects(project_config, &mut sys);

    // 处理用户命令
    process_commands(&mut logs, &mut sys);
}

fn start_projects(project_config: Vec<ProjectConfig>, sys: &mut System) -> HashMap<String, log::NodeLog> {
    project_config
        .into_iter()
        .map(|project| {
            start_project_and_log(project, sys)
        })
        // * 将迭代器转化为HashMap collect可以根据标注类型生成数据
        .collect()
}

fn process_commands(logs: &mut HashMap<String, log::NodeLog>, sys: &mut System) {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            match l.trim() {
                "exit" => {
                    exit_program(logs, sys);
                    break;
                }
                // * 匹配模式中的模式守卫
                // * cmd 是一个变量绑定 它捕获的是l.trim()的值
                // * if cmd.ends_with("exit") 是一个模式守卫 当这个守卫的条件为true的时候才会匹配成功
                cmd if cmd.starts_with("exit") => {
                    stop_project(logs, cmd);
                    continue;
                }
                cmd if cmd.starts_with("restart") => {
                    restart_project(logs, cmd, sys);
                    continue;
                }
                _ => {}
            }
        }
    }
}

fn start_project_and_log(project: ProjectConfig, sys: &mut System) -> (String, log::NodeLog) {
    let name = project.name.clone();
    let mut project = project::Project::new(project);

    println!("正在启动: {:?}...", project);
    project.start(sys).expect(&format!("启动项目 {} 失败", name));
    println!("{} 启动成功", name);

    // * 之前的问题存在于project的所有权 如果把for循环内部的project的可变引用交给log for循环结束后project会被销毁 导致了悬垂引用
    // * for循环还会消耗project_hashmap
    let mut log = log::NodeLog::new(project);
    log.log_start();
    print_sys_info(sys);

    (name, log)
}

fn print_sys_info(sys: &mut System) {
    sys.refresh_memory();
    println!("剩余内存: {:?}MB", sys.free_memory() / 1024 / 1024);
    println!("总共内存: {:?}MB", sys.total_memory() / 1024 / 1024);
}

fn stop_project(logs: &mut HashMap<String, log::NodeLog>, cmd: &str) {
    let project_name: &str = cmd.trim_start_matches("exit").trim();
    println!("{}", project_name);
    if let Some(log) = logs.get_mut(project_name) {
        log.project.stop().expect(&format!("停止项目 {} 失败", project_name));
        println!("{} 已停止", project_name);
    } else {
        println!("{} 不存在", project_name);
    }
}

fn restart_project(logs: &mut HashMap<String, log::NodeLog>, cmd: &str, sys: &mut System) {
    let project_name: &str = cmd.trim_start_matches("restart").trim();
    println!("{}", project_name);
    if let Some(log) = logs.get_mut(project_name) {
        if let project::Status::Running = log.project.status {
            log.project.stop().expect(&format!("停止项目 {} 失败", project_name));
        }
        log.project.start(sys).expect(&format!("重新启动项目 {} 失败", project_name));
        log.log_start();
        println!("{} 已重新启动", project_name);
    } else {
        println!("{} 不存在", project_name);
    }
}

fn exit_program(logs: &mut HashMap<String, log::NodeLog>, sys: &mut System) {
    logs.iter_mut().for_each(|(_, log)| {
        log.project.stop().expect(&format!("停止项目 {} 失败", log.project.project_info.name));
    });
    print_sys_info(sys);
}
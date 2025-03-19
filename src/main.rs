mod project;
mod config;
mod log;

use std::collections::HashMap;
use std::io::*;
use sysinfo::System;

fn main() {
    // * 测试启动

    let project_config = config::ReadConfig::new();
    let mut sys = System::new();

    sys.refresh_memory();
    println!("剩余内存: {:?}MB", sys.free_memory() / 1024 / 1024);

    println!("project_config: {:?}", project_config);

    let mut logs: HashMap<String, log::NodeLog> = project_config
        .into_iter()
        .map(|project| {
            let name = project.name.clone();
            let mut project = project::Project::new(project);

            println!("正在启动: {:?}...", project);
            project.start().expect(&format!("启动项目 {} 失败", name));
            println!("{} 启动成功", name);

            // * 之前的问题存在于project的所有权 如果把for循环内部的project的可变引用交给log for循环结束后project会被销毁 导致了悬垂引用
            // * for循环还会消耗project_hashmap
            let mut log = log::NodeLog::new(project);
            log.log_start();

            (name, log)
        })
        // * 将迭代器转化为HashMap collect可以根据标注类型生成数据
        .collect();

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            match l.trim() {
                "exit" => {
                    logs.iter_mut().for_each(|(_, log)| {
                        log.project.stop().expect(&format!("停止项目 {} 失败", log.project.name));
                    });
                    break;
                }
                // * 匹配模式中的模式守卫
                // * cmd 是一个变量绑定 它捕获的是l.trim()的值
                // * if cmd.ends_with("exit") 是一个模式守卫 当这个守卫的条件为true的时候才会匹配成功
                cmd if cmd.ends_with("exit") => {
                    let project_name = cmd.trim_end_matches("exit");
                    if let Some(log) = logs.get_mut(project_name) {
                        log.project.stop().expect(&format!("停止项目 {} 失败", project_name));
                        println!("{} 已停止", project_name);
                    } else {
                        println!("{} 不存在", project_name);
                    }
                }
                cmd if cmd.starts_with("restart") => {
                    let project_name: &str = cmd.trim_start_matches("restart");
                    if let Some(log) = logs.get_mut(project_name) {
                        if let project::Status::Running = log.project.status {
                            log.project.stop().expect(&format!("停止项目 {} 失败", project_name));
                        }
                        log.project.start().expect(&format!("重新启动项目 {} 失败", project_name));
                        println!("{} 已重新启动", project_name);
                    } else {
                        println!("{} 不存在", project_name);
                    }
                }
                _ => {}
            }
        }
    }

    println!("程序已退出");
}

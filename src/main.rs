mod project;
mod config;
mod log;

use std::collections::HashMap;
use std::io::*;

fn main() {
    // * 测试启动

    let project_config = config::ReadConfig::new();

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
            if l.trim() == "exit" {
                logs.iter_mut().for_each(|(_, log)| {
                    log.project.stop().expect(&format!("停止项目 {} 失败", log.project.name));
                });
                break;
            }
        }
    }

    println!("程序已退出");
}

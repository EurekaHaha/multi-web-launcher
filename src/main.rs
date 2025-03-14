mod project;
mod config;
mod log;

use std::io::*;

fn main() {
    // * 测试启动

    let project_config = config::ProjectConfig::new();
    // 创建 proxy 项目实例
    let mut proxy = project::Project::new(&project_config);

    // 启动 proxy
    println!("正在启动 proxy...");
    proxy.start().expect("启动 proxy 失败");
    println!("proxy 启动成功");

    let mut proxy_log: log::NodeLog<'_> = log::NodeLog::new(&mut proxy);
    proxy_log.log_start();

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            if l.trim() == "exit" {
                proxy.stop().expect("停止 proxy 失败");
                break;
            }
        }
    }

    println!("程序已退出");

}
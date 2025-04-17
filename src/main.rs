mod project;
mod config;
mod log;
mod runtime;

use sysinfo::System;


fn main() {

    let project_config = config::ReadConfig::new();
    println!("project_config: {:?}", project_config);

    let mut sys = System::new();
    runtime::print_sys_info(&mut sys);
    // 启动所有项目并监控日志
    let mut logs = runtime::start_projects(project_config, &mut sys);

    // 处理用户命令
    runtime::process_commands(&mut logs, &mut sys);
}

use std::env;
use sysinfo::{Pid, System};

pub fn detect_shell_environment() -> Option<(String, &'static str)> {
    let os = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "MacOS"
    } else {
        return None;
    };

    if let Ok(shell) = env::var("SHELL") {
        if let Some(shell) = shell.split('/').last() {
            return Some((shell.into(), os));
        } else {
            return None;
        }
    }

    let mut system = System::new();
    system.refresh_all();
    let pid = Pid::from(std::process::id() as usize);

    let mut shell = system
        .process(pid)
        .and_then(|p| p.parent())
        .and_then(|p| system.process(p))
        .and_then(|p| p.name().to_str())?;

    if cfg!(target_os = "windows") {
        shell = shell.trim_end_matches(".exe");
    }

    Some((shell.into(), os))
}

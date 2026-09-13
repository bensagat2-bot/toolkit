use std::process::Command;

use super::utils;

fn base_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

fn adb_path() -> String {
    utils::adb_path()
}

fn fastboot_path() -> String {
    utils::fastboot_path()
}

fn is_bundled(path: &str) -> bool {
    std::path::Path::new(path).is_absolute()
}

pub async fn run_adb(args: &[&str]) -> String {
    let path = adb_path();
    if is_bundled(&path) && !std::path::Path::new(&path).exists() {
        return "adb: embedded platform-tools missing (reinstall the toolkit)".into();
    }
    let mut cmd = base_command(&path);
    cmd.args(args);
    let output = super::utils::run_output(&mut cmd, 8000);
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub async fn detect() -> (String, Option<String>) {
    let adb_output = run_adb(&["devices"]).await;
    for line in adb_output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            return ("adb".into(), Some(parts[0].into()));
        }
    }

    let fb_output = run_fastboot(&["devices"]).await;
    for line in fb_output.lines() {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if !parts.is_empty() && !line.contains("waiting") {
            return ("fastboot".into(), Some(parts[0].into()));
        }
    }

    ("none".into(), None)
}

pub async fn run_fastboot(args: &[&str]) -> String {
    let path = fastboot_path();
    if is_bundled(&path) && !std::path::Path::new(&path).exists() {
        return "fastboot: embedded platform-tools missing (reinstall the toolkit)".into();
    }
    let mut cmd = base_command(&path);
    cmd.args(args);
    let output = super::utils::run_output(&mut cmd, 8000);
    String::from_utf8_lossy(&output.stdout).to_string()
}

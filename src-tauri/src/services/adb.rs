use std::process::Command;

fn base_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

pub async fn run_adb(args: &[&str]) -> String {
    let output = base_command("adb")
        .args(args)
        .output()
        .unwrap_or_else(|_| std::process::Output {
            stdout: vec![],
            stderr: format!("adb: command not found").into_bytes(),
            status: std::process::ExitStatus::default(),
        });
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
    let output = base_command("fastboot")
        .args(args)
        .output()
        .unwrap_or_else(|_| std::process::Output {
            stdout: vec![],
            stderr: format!("fastboot: command not found").into_bytes(),
            status: std::process::ExitStatus::default(),
        });
    String::from_utf8_lossy(&output.stdout).to_string()
}

use std::process::Command;

fn run_cmd(program: &str, args: &[&str]) -> String {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.args(args);
    cmd.output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

#[cfg(target_os = "windows")]
pub fn get_hwid() -> String {
    let out = run_cmd("wmic", &[
        "cpu", "get", "processorid", "/format:list",
    ]);
    out.lines()
        .find_map(|l| l.trim().strip_prefix("ProcessorId=").map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "UNKNOWN-HWID".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn get_hwid() -> String {
    let out = run_cmd("hostname", &[]);
    if out.is_empty() { "UNKNOWN-HWID".to_string() } else { out }
}

#[cfg(target_os = "windows")]
pub fn get_device_model() -> String {
    let out = run_cmd("wmic", &[
        "computersystem", "get", "manufacturer,model", "/format:list",
    ]);
    let mut manufacturer = String::new();
    let mut model = String::new();
    for line in out.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("Manufacturer=") {
            manufacturer = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Model=") {
            model = v.trim().to_string();
        }
    }
    if manufacturer.is_empty() && model.is_empty() {
        return "Unknown device".to_string();
    }
    if manufacturer.is_empty() {
        return model;
    }
    if model.is_empty() {
        return manufacturer;
    }
    format!("{manufacturer} {model}")
}

#[cfg(not(target_os = "windows"))]
pub fn get_device_model() -> String {
    "Unknown device".to_string()
}
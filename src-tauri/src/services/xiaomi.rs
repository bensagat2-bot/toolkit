use serde::{Deserialize, Serialize};

use crate::services::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub mode: String,
    pub serial: Option<String>,
    pub model: String,
    pub brand: String,
    pub android: String,
    pub bootloader: String,
    pub root_status: String,
    pub product: Option<String>,
}

fn adb(args: &[&str], timeout_ms: u64) -> String {
    let path = utils::adb_path();
    utils::run_cmd(&path, args, timeout_ms)
}

fn fastboot(args: &[&str], timeout_ms: u64) -> String {
    let path = utils::fastboot_path();
    utils::run_cmd(&path, args, timeout_ms)
}

pub fn detect_device() -> DeviceInfo {
    let mut info = DeviceInfo {
        mode: "none".into(),
        serial: None,
        model: "N/A".into(),
        brand: "N/A".into(),
        android: "N/A".into(),
        bootloader: "N/A".into(),
        root_status: "Not Rooted".into(),
        product: None,
    };

    // Try ADB first
    let adb_out = adb(&["devices"], 5000);
    for line in adb_out.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            let serial = parts[0].to_string();
            info.mode = "adb".into();
            info.serial = Some(serial.clone());
            info.model = adb_prop(&serial, "ro.product.model");
            info.brand = adb_prop(&serial, "ro.product.brand");
            info.android = format!(
                "{} (API {})",
                adb_prop(&serial, "ro.build.version.release"),
                adb_prop(&serial, "ro.build.version.sdk")
            );
            info.bootloader = check_bootloader_adb(&serial);
            info.root_status = check_root_adb(&serial);
            return info;
        }
        if parts.len() >= 2 && parts[1] == "unauthorized" {
            info.mode = "unauthorized".into();
            return info;
        }
    }

    // Try fastboot
    let fb_out = fastboot(&["devices"], 5000);
    for line in fb_out.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.contains("fastboot") {
            let serial = trimmed.split_whitespace().next().unwrap_or("").to_string();
            info.mode = "fastboot".into();
            info.serial = Some(serial);
            info.product = Some(get_fastboot_var("product"));
            info.model = info.product.clone().unwrap_or_default();
            info.brand = get_fastboot_var("manufacturer");
            info.bootloader = check_bootloader_fastboot();
            return info;
        }
    }

    info
}

fn adb_prop(serial: &str, prop: &str) -> String {
    let out = adb(&["-s", serial, "shell", "getprop", prop], 5000);
    let val = out.trim().to_string();
    if val.is_empty() || val == "unknown" || val == "null" {
        "N/A".into()
    } else {
        val
    }
}

fn get_fastboot_var(var: &str) -> String {
    let out = fastboot(&["getvar", var], 5000);
    let needle = format!("{}:", var.to_lowercase());
    for line in out.lines() {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find(&needle) {
            let val = line[idx + needle.len()..].trim();
            if !val.is_empty() {
                return val.to_string();
            }
        }
    }
    "N/A".into()
}

fn check_bootloader_adb(serial: &str) -> String {
    let vbs = adb(
        &["-s", serial, "shell", "getprop", "ro.boot.verifiedbootstate"],
        5000,
    );
    let vbs = vbs.trim().to_lowercase();
    match vbs.as_str() {
        "orange" | "yellow" => "Unlocked".into(),
        "green" => "Locked".into(),
        _ => {
            let lock = adb(
                &["-s", serial, "shell", "getprop", "ro.secureboot.lockstate"],
                5000,
            );
            let lock = lock.trim().to_lowercase();
            match lock.as_str() {
                "unlocked" => "Unlocked".into(),
                "locked" => "Locked".into(),
                _ => "Unknown".into(),
            }
        }
    }
}

fn check_bootloader_fastboot() -> String {
    let out = fastboot(&["getvar", "unlocked"], 5000);
    let lower = out.to_lowercase();
    if let Some(val) = lower.split("unlocked:").nth(1) {
        let v = val.trim_start();
        if v.starts_with('y') {
            return "Unlocked".into();
        }
        if v.starts_with('n') {
            return "Locked".into();
        }
    }
    "Unknown".into()
}

fn check_root_adb(serial: &str) -> String {
    let which = adb(&["-s", serial, "shell", "which", "su"], 5000);
    if !which.trim().is_empty() && !which.contains("not found") && !which.contains("No such file") {
        return "Rooted (su)".into();
    }
    let id = adb(&["-s", serial, "shell", "su", "-c", "id"], 5000);
    if id.contains("uid=0") {
        return "Rooted (working)".into();
    }
    let magisk = adb(&["-s", serial, "shell", "magisk", "-c"], 5000);
    let magisk_trimmed = magisk.trim();
    if !magisk_trimmed.is_empty() && !magisk_trimmed.contains("not found") && !magisk_trimmed.contains("No such file") && !magisk_trimmed.contains("Permission denied") {
        return "Rooted (Magisk)".into();
    }
    let ksu = adb(&["-s", serial, "shell", "ksud", "--version"], 5000);
    let ksu_trimmed = ksu.trim();
    if !ksu_trimmed.is_empty() && !ksu_trimmed.contains("not found") && !ksu_trimmed.contains("No such file") {
        return "Rooted (KernelSU)".into();
    }
    "Not Rooted".into()
}

pub fn get_device_token() -> Result<String, String> {
    let out = fastboot(&["oem", "get_token"], 15000);
    for line in out.lines() {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find("token:") {
            let val = line[idx + "token:".len()..].trim();
            if !val.is_empty() {
                return Ok(val.to_string());
            }
        }
    }
    Err("Could not read device token".into())
}

pub fn get_device_product() -> Result<String, String> {
    let out = fastboot(&["getvar", "product"], 10000);
    for line in out.lines() {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find("product:") {
            let val = line[idx + "product:".len()..].trim();
            if !val.is_empty() {
                return Ok(val.to_string());
            }
        }
    }
    Err("Could not read device product".into())
}

pub fn fastboot_stage(file: &str) -> Result<String, String> {
    let out = fastboot(&["stage", file], 120000);
    if out.contains("FAILED") || out.contains("error") {
        return Err(out);
    }
    Ok(out)
}

pub fn fastboot_oem_unlock() -> Result<String, String> {
    let out = fastboot(&["oem", "unlock"], 60000);
    if out.contains("FAILED") || out.contains("error") {
        return Err(out);
    }
    Ok(out)
}

pub fn fastboot_reboot() -> Result<String, String> {
    let out = fastboot(&["reboot"], 30000);
    Ok(out)
}

pub fn adb_reboot_bootloader(serial: &str) -> Result<String, String> {
    let out = adb(&["-s", serial, "reboot", "bootloader"], 30000);
    Ok(out)
}

/// Finds the connected ADB device and reboots it to fastboot mode.
pub fn reboot_to_bootloader() -> Result<String, String> {
    let out = adb(&["devices"], 5000);
    let mut serial = String::new();
    for line in out.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            serial = parts[0].to_string();
            break;
        }
        if parts.len() >= 2 && parts[1] == "unauthorized" {
            return Err("USB debugging is not authorized. Unlock your phone and tap 'Allow'.".into());
        }
    }
    if serial.is_empty() {
        return Err("No ADB device found. Enable USB debugging and connect.".into());
    }
    adb_reboot_bootloader(&serial)
}

pub fn check_fastboot() -> bool {
    let out = fastboot(&["--version"], 5000);
    !out.trim().is_empty() && !out.contains("not found")
}

pub fn check_adb() -> bool {
    let out = adb(&["version"], 5000);
    !out.trim().is_empty() && !out.contains("not found")
}

pub fn run_fastboot_cmd(args: Vec<String>) -> String {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    fastboot(&arg_refs, 120000)
}

pub fn run_adb_cmd(args: Vec<String>) -> String {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    adb(&arg_refs, 120000)
}

pub fn scan_rom_folder(folder: &str) -> Result<serde_json::Value, String> {
    let dir = std::path::Path::new(folder);
    if !dir.is_dir() {
        return Err("Selected path is not a folder".into());
    }

    let mut commands = Vec::new();
    let mut rom_type = "unknown";

    // Check for flash_all.bat (MiFlash)
    for entry in std::fs::read_dir(dir).map_err(|e| format!("Failed to read folder: {e}"))? {
        let entry = entry.map_err(|e| format!("Read error: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();

        if lower.starts_with("flash_all") && lower.ends_with(".bat") {
            rom_type = "miflash";
            let bat_path = entry.path();
            let content = std::fs::read_to_string(&bat_path)
                .map_err(|e| format!("Failed to read bat: {e}"))?;
            for line in content.lines() {
                let line = line.trim().to_string();
                if line.starts_with("fastboot") {
                    let cleaned = line.replace("%~dp0", "").replace('"', "").replace('\\', "/");
                    let toks: Vec<&str> = cleaned.split_whitespace().collect();
                    if toks.len() >= 3 && toks[1] == "flash" {
                        commands.push(serde_json::json!({
                            "type": "flash",
                            "partition": toks[2],
                            "args": toks[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                        }));
                    } else if toks.len() >= 2 && ["erase", "reboot", "set_active"].contains(&toks[1]) {
                        commands.push(serde_json::json!({
                            "type": toks[1],
                            "partition": toks.get(2).unwrap_or(&""),
                            "args": toks[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                        }));
                    }
                }
            }
            break;
        }

        if lower == "scatter.xml" {
            rom_type = "mtk";
            break;
        }

        if lower.starts_with("rawprogram") && lower.ends_with(".xml") {
            rom_type = "qcom";
            break;
        }
    }

    Ok(serde_json::json!({
        "rom_type": rom_type,
        "commands": commands,
    }))
}

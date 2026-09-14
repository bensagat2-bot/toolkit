// ── Debloat ─────────────────────────────────────────────
// Lists installed packages with app labels, system/user detection.
// Auto-grants SU via ensure_su_access before operations.

use serde::Serialize;
use super::utils::{run_cmd, adb_path};

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub package_name: String,
    pub app_name: String,
    pub is_system: bool,
    pub is_disabled: bool,
}

fn adb(args: &[&str], timeout_ms: u64) -> String {
    let path = adb_path();
    run_cmd(&path, args, timeout_ms)
}

fn adb_serial(serial: &str, args: &[&str], timeout_ms: u64) -> String {
    let path = adb_path();
    let mut full = vec!["-s", serial];
    full.extend_from_slice(args);
    run_cmd(&path, &full, timeout_ms)
}

fn detect_serial() -> Option<String> {
    let out = adb(&["devices"], 5000);
    for line in out.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            return Some(parts[0].into());
        }
    }
    None
}

fn parse_package_list(output: &str) -> Vec<String> {
    let mut packages = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with("package:") { continue; }
        if let Some(eq_pos) = line.find('=') {
            let pkg = line[eq_pos + 1..].trim().to_string();
            packages.push(pkg);
        }
    }
    packages
}

fn parse_disabled_set(output: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for line in output.lines() {
        let l = line.trim();
        if l.starts_with("package:") {
            if let Some(eq) = l.find('=') {
                set.insert(l[eq + 1..].trim().to_string());
            }
        }
    }
    set
}

fn extract_app_label_from_dump(dump: &str) -> Option<String> {
    for line in dump.lines() {
        let t = line.trim();
        if t.starts_with("application-label:") {
            let label = t.split(':').nth(1).unwrap_or("").trim().trim_matches(|c| c == '\'' || c == '"');
            if !label.is_empty() { return Some(label.to_string()); }
        }
        if let Some(start) = t.find("android:label=") {
            let val = t[start + 14..].trim().trim_matches(|c| c == '\'' || c == '"');
            if !val.is_empty() && val != "null" && val != "@null" {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Lists all installed packages with app names and system/user status.
pub fn list_packages() -> Result<Vec<AppInfo>, String> {
    let serial = detect_serial().ok_or("No ADB device detected")?;

    // Auto-grant SU before listing
    let _ = crate::services::utils::ensure_su_access();

    let sys_output = adb_serial(&serial, &["shell", "pm", "list", "packages", "-f", "-s"], 15000);
    let user_output = adb_serial(&serial, &["shell", "pm", "list", "packages", "-f", "-3"], 15000);
    let disabled_output = adb_serial(&serial, &["shell", "pm", "list", "packages", "-d"], 10000);

    let sys_pkgs = parse_package_list(&sys_output);
    let user_pkgs = parse_package_list(&user_output);
    let disabled_set = parse_disabled_set(&disabled_output);

    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();

    for pkg in &sys_pkgs {
        if seen.contains(pkg) { continue; }
        seen.insert(pkg.clone());
        let dump = adb_serial(&serial, &["shell", "dumpsys", "package", pkg], 8000);
        let app_name = extract_app_label_from_dump(&dump).unwrap_or_else(|| pkg.clone());
        apps.push(AppInfo {
            package_name: pkg.clone(),
            app_name,
            is_system: true,
            is_disabled: disabled_set.contains(pkg),
        });
    }

    for pkg in &user_pkgs {
        if seen.contains(pkg) { continue; }
        seen.insert(pkg.clone());
        let dump = adb_serial(&serial, &["shell", "dumpsys", "package", pkg], 8000);
        let app_name = extract_app_label_from_dump(&dump).unwrap_or_else(|| pkg.clone());
        apps.push(AppInfo {
            package_name: pkg.clone(),
            app_name,
            is_system: false,
            is_disabled: disabled_set.contains(pkg),
        });
    }

    apps.sort_by(|a, b| a.app_name.cmp(&b.app_name));
    Ok(apps)
}

pub fn uninstall_package(package_name: String) -> Result<String, String> {
    let serial = detect_serial().ok_or("No ADB device detected")?;

    let result = adb_serial(&serial, &["shell", "pm", "uninstall", "--user", "0", &package_name], 15000);
    if result.contains("Success") || result.contains("success") {
        return Ok(format!("Uninstalled: {package_name}"));
    }

    let result2 = adb_serial(&serial, &["shell", "pm", "disable", "--user", "0", &package_name], 10000);
    if result2.contains("disabled") || result2.contains("success") || result2.contains("new state") {
        return Ok(format!("Disabled: {package_name}"));
    }

    let result3 = adb_serial(&serial, &["shell", "su", "-c", &format!("pm uninstall --user 0 {package_name}")], 15000);
    if result3.contains("Success") || result3.contains("success") {
        return Ok(format!("Uninstalled (root): {package_name}"));
    }

    Err(format!("Failed to remove {package_name}"))
}

pub fn enable_package(package_name: String) -> Result<String, String> {
    let serial = detect_serial().ok_or("No ADB device detected")?;
    let result = adb_serial(&serial, &["shell", "pm", "enable", "--user", "0", &package_name], 10000);
    if result.contains("enabled") || result.contains("success") || result.contains("new state") {
        Ok(format!("Enabled: {package_name}"))
    } else {
        Err(format!("Failed to enable {package_name}: {result}"))
    }
}
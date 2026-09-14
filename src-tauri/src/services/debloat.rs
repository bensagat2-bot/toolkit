// ── Debloat ─────────────────────────────────────────────
// Lists installed packages with app labels, system/user detection.
// Auto-grants SU via ensure_su_access before operations.

use serde::Serialize;
use super::utils::{run_cmd, adb_path};

use std::collections::{HashMap, HashSet};

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

fn parse_package_list_from_paths(output: &str) -> Vec<(String, String)> {
    let mut packages = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with("package:") { continue; }
        let rest = &line["package:".len()..];
        if let Some(eq_pos) = rest.find('=') {
            let apk_path = rest[..eq_pos].to_string();
            let pkg = rest[eq_pos + 1..].trim().to_string();
            let app_name = extract_app_name_from_apk_path(&apk_path);
            packages.push((pkg, app_name));
        }
    }
    packages
}

fn extract_app_name_from_apk_path(path: &str) -> String {
    let stem = path.rsplit('/').next()
        .and_then(|f| f.strip_suffix(".apk"))
        .unwrap_or("");
    if stem.is_empty() { return String::new() }
    let mut result = String::new();
    let mut prev_upper = false;
    for (i, c) in stem.chars().enumerate() {
        if i > 0 && c.is_uppercase() && !prev_upper && !result.ends_with(' ') {
            result.push(' ');
        }
        prev_upper = c.is_uppercase();
        if c == '_' || c == '-' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_disabled_set(output: &str) -> HashSet<String> {
    let mut set = HashSet::new();
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

fn fetch_labels_batch(serial: &str, pkgs: &[String]) -> HashMap<String, String> {
    let mut label_map = HashMap::new();
    let out = adb_serial(&serial, &["shell", "dumpsys", "package", "packages"], 30000);
    let mut current_pkg = String::new();
    for line in out.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Package [") {
            if let Some(bracket) = rest.find(']') {
                current_pkg = rest[..bracket].to_string();
            }
        }
        if t.starts_with("application-label:") {
            let label = t.split(':').nth(1).unwrap_or("").trim().trim_matches(|c| c == '\'' || c == '"');
            if !label.is_empty() && !current_pkg.is_empty() && pkgs.contains(&current_pkg) {
                label_map.insert(current_pkg.clone(), label.to_string());
            }
        }
    }
    label_map
}

/// Lists all installed packages with app names and system/user status.
pub fn list_packages() -> Result<Vec<AppInfo>, String> {
    let serial = detect_serial().ok_or("No ADB device detected")?;

    let _ = crate::services::utils::ensure_su_access();

    let sys_output = adb_serial(&serial, &["shell", "pm", "list", "packages", "-f", "-s"], 15000);
    let user_output = adb_serial(&serial, &["shell", "pm", "list", "packages", "-f", "-3"], 15000);
    let disabled_output = adb_serial(&serial, &["shell", "pm", "list", "packages", "-d"], 10000);

    let sys_pkgs = parse_package_list_from_paths(&sys_output);
    let user_pkgs = parse_package_list_from_paths(&user_output);
    let disabled_set = parse_disabled_set(&disabled_output);

    let mut all_package_names: Vec<String> = sys_pkgs.iter().chain(user_pkgs.iter())
        .map(|(pkg, _)| pkg.clone()).collect();
    all_package_names.sort();
    all_package_names.dedup();

    let dump_labels = fetch_labels_batch(&serial, &all_package_names);

    let mut seen = HashSet::new();
    let mut apps = Vec::new();

    for (pkg, fallback_name) in &sys_pkgs {
        if seen.contains(pkg) { continue; }
        seen.insert(pkg.clone());
        let app_name = dump_labels.get(pkg).cloned()
            .or_else(|| if !fallback_name.is_empty() { Some(fallback_name.clone()) } else { None })
            .unwrap_or_else(|| pkg.clone());
        apps.push(AppInfo {
            package_name: pkg.clone(),
            app_name,
            is_system: true,
            is_disabled: disabled_set.contains(pkg),
        });
    }

    for (pkg, fallback_name) in &user_pkgs {
        if seen.contains(pkg) { continue; }
        seen.insert(pkg.clone());
        let app_name = dump_labels.get(pkg).cloned()
            .or_else(|| if !fallback_name.is_empty() { Some(fallback_name.clone()) } else { None })
            .unwrap_or_else(|| pkg.clone());
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

    let result2 = adb_serial(&serial, &["shell", "cmd", "package", "uninstall", "--user", "0", &package_name], 15000);
    if result2.contains("Success") || result2.contains("success") {
        return Ok(format!("Uninstalled: {package_name}"));
    }

    let result3 = adb_serial(&serial, &["shell", "pm", "disable-user", "--user", "0", &package_name], 10000);
    if result3.contains("disabled") || result3.contains("success") || result3.contains("new state") {
        return Ok(format!("Disabled: {package_name}"));
    }

    let result4 = adb_serial(&serial, &["shell", "su", "-c", &format!("pm uninstall --user 0 {}", package_name)], 15000);
    if result4.contains("Success") || result4.contains("success") {
        return Ok(format!("Uninstalled (root): {package_name}"));
    }

    let result5 = adb_serial(&serial, &["shell", "su", "-c", &format!("cmd package uninstall --user 0 {}", package_name)], 15000);
    if result5.contains("Success") || result5.contains("success") {
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
        let result2 = adb_serial(&serial, &["shell", "cmd", "package", "install-existing", "--user", "0", &package_name], 10000);
        if result2.contains("Success") || result2.contains("success") || result2.contains("installing") {
            Ok(format!("Restored: {package_name}"))
        } else {
            Err(format!("Failed to enable {package_name}: {result}"))
        }
    }
}
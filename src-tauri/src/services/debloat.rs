// ── Debloat ─────────────────────────────────────────────
// Lists installed packages with app labels, system/user detection.
// Auto-grants SU via ensure_su_access before operations.

use base64::Engine;

use serde::Serialize;
use super::utils::{run_cmd, adb_path};

use std::collections::{HashMap, HashSet};
use std::io::Read;

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
        if let Some(eq_pos) = rest.rfind('=') {
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
            if let Some(eq) = l.rfind('=') {
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

    // 1) Standard adb uninstall (user apps)
    let r1 = adb_serial(&serial, &["uninstall", &package_name], 15000);
    if !r1.contains("Failure") && !r1.contains("Failed") {
        return Ok(format!("Uninstalled: {package_name}"));
    }

    // 2) pm uninstall --user 0 (remove for current user, works on some system apps)
    let r2 = adb_serial(&serial, &["shell", "pm", "uninstall", "--user", "0", &package_name], 15000);
    if !r2.contains("Failure") && !r2.contains("Failed") {
        return Ok(format!("Uninstalled: {package_name}"));
    }

    // 3) cmd package uninstall (alternate API)
    let r3 = adb_serial(&serial, &["shell", "cmd", "package", "uninstall", "--user", "0", &package_name], 15000);
    if !r3.contains("Failure") && !r3.contains("Failed") {
        return Ok(format!("Uninstalled: {package_name}"));
    }

    // 4) disable-user (fallback for system apps that refuse uninstall)
    let r4 = adb_serial(&serial, &["shell", "pm", "disable-user", "--user", "0", &package_name], 10000);
    if r4.contains("disabled") || r4.contains("new state") {
        return Ok(format!("Disabled: {package_name}"));
    }

    // 5) Root attempts
    let r5 = adb_serial(&serial, &["shell", "su", "-c", &format!("pm uninstall --user 0 {}", package_name)], 15000);
    if !r5.contains("Failure") && !r5.contains("Failed") && !r5.contains("Error") {
        return Ok(format!("Uninstalled (root): {package_name}"));
    }

    let r6 = adb_serial(&serial, &["shell", "su", "-c", &format!("cmd package uninstall --user 0 {}", package_name)], 15000);
    if !r6.contains("Failure") && !r6.contains("Failed") && !r6.contains("Error") {
        return Ok(format!("Uninstalled (root): {package_name}"));
    }

    Err(format!("Failed to remove {package_name}"))
}

/// Preferred icon entry paths inside an APK (highest density first).
const ICON_PATHS: &[&str] = &[
    "res/mipmap-xxxhdpi-v4/ic_launcher_foreground.png",
    "res/mipmap-xxxhdpi-v4/ic_launcher.png",
    "res/mipmap-xxhdpi-v4/ic_launcher_foreground.png",
    "res/mipmap-xxhdpi-v4/ic_launcher.png",
    "res/mipmap-xxxhdpi/ic_launcher_foreground.png",
    "res/mipmap-xxxhdpi/ic_launcher.png",
    "res/mipmap-xxhdpi/ic_launcher_foreground.png",
    "res/mipmap-xxhdpi/ic_launcher.png",
    "res/mipmap-xhdpi-v4/ic_launcher_foreground.png",
    "res/mipmap-xhdpi-v4/ic_launcher.png",
    "res/mipmap-xhdpi/ic_launcher.png",
    "res/mipmap-hdpi-v4/ic_launcher.png",
    "res/mipmap-hdpi/ic_launcher.png",
    "res/mipmap-mdpi/ic_launcher.png",
    "res/mipmap-xxxhdpi-v4/ic_launcher_round.png",
    "res/mipmap-xxhdpi-v4/ic_launcher_round.png",
    "res/mipmap-xxxhdpi-v4/ic_launcher_round_foreground.png",
    "res/drawable-xxxhdpi-v4/ic_launcher.png",
    "res/drawable-xxhdpi-v4/ic_launcher.png",
    "res/drawable-xxhdpi/ic_launcher.png",
    "res/drawable-xhdpi/ic_launcher.png",
    "res/drawable/ic_launcher.png",
    "res/drawable/icon.png",
    "res/drawable/app_icon.png",
    "res/mipmap-xxxhdpi-v4/ic_launcher_foreground.webp",
    "res/mipmap-xxxhdpi-v4/ic_launcher.webp",
    "res/mipmap-xxhdpi-v4/ic_launcher_foreground.webp",
    "res/mipmap-xxhdpi-v4/ic_launcher.webp",
    "res/mipmap-xxxhdpi/ic_launcher.webp",
    "res/mipmap-xxhdpi/ic_launcher.webp",
    "res/mipmap-xhdpi-v4/ic_launcher.webp",
    "res/drawable-xxxhdpi-v4/ic_launcher.webp",
    "res/drawable/ic_launcher.webp",
];

/// Extract launcher icon for one package by pulling the APK and reading it as ZIP.
/// Returns "data:image/png;base64,..." or "data:image/webp;base64,..." or empty on failure.
pub fn get_icon(package_name: &str) -> Result<String, String> {
    let serial = detect_serial().ok_or("No ADB device detected")?;

    // 1) Get APK path on device
    let out = adb_serial(&serial, &["shell", "pm", "path", package_name], 8000);
    let mut apk_path = String::new();
    for line in out.lines() {
        let s = line.trim();
        let path = if let Some(rest) = s.strip_prefix("package:") { rest } else { s };
        if path.ends_with(".apk") && (path.contains("base.apk") || apk_path.is_empty()) {
            apk_path = path.to_string();
            if path.contains("base.apk") { break; }
        }
    }
    if apk_path.is_empty() {
        return Err("APK path not found".to_string());
    }

    // 2) Pull APK to temp file
    let tmp_dir = std::env::temp_dir();
    let tmp_apk = tmp_dir.join(format!("v1per_icon_{}.apk", package_name.replace('.', "_")));
    let tmp_str = tmp_apk.to_string_lossy().to_string();
    let pull = adb_serial(&serial, &["pull", &apk_path, &tmp_str], 120000);
    if !pull.contains("file") && !pull.contains("bytes") && !pull.trim().is_empty() && !pull.contains("pulled") {
        let _ = std::fs::remove_file(&tmp_apk);
        return Err(format!("adb pull failed: {pull}"));
    }
    if !tmp_apk.exists() || tmp_apk.metadata().map(|m| m.len()).unwrap_or(0) < 100 {
        let _ = std::fs::remove_file(&tmp_apk);
        return Err("APK too small or missing after pull".to_string());
    }

    // 3) Open as ZIP and search for icon
    let result = extract_icon_from_apk(&tmp_apk);

    // 4) Cleanup temp file
    let _ = std::fs::remove_file(&tmp_apk);

    result
}

fn extract_icon_from_apk(apk_path: &std::path::Path) -> Result<String, String> {
    let file = std::fs::File::open(apk_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid APK/ZIP: {e}"))?;

    // 1) Try preferred paths first
    for path in ICON_PATHS {
        if let Ok(mut entry) = archive.by_name(path) {
            let size = entry.size();
            if size >= 100 && size <= 900_000 {
                let mut data = Vec::new();
                if entry.read_to_end(&mut data).is_ok() && data.len() >= 50 {
                    let mime = if path.ends_with(".webp") { "image/webp" } else { "image/png" };
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    return Ok(format!("data:{};base64,{}", mime, b64));
                }
            }
        }
    }

    // 2) Fallback: scan all entries for best icon match
    let mut candidates: Vec<(String, Vec<u8>, u32)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.name().to_lowercase();
        if !name.starts_with("res/") { continue; }
        if !name.ends_with(".png") && !name.ends_with(".webp") { continue; }
        let size = entry.size();
        if size < 300 || size > 900_000 { continue; }
        if name.contains("background") { continue; }

        let score = score_icon_path(&name, size);
        if score == 0 { continue; }

        let mut data = Vec::new();
        if entry.read_to_end(&mut data).is_err() { continue; }
        candidates.push((entry.name().to_string(), data, score));
    }

    candidates.sort_by(|a, b| b.2.cmp(&a.2));
    if let Some((path, data, _)) = candidates.into_iter().next() {
        let mime = if path.ends_with(".webp") { "image/webp" } else { "image/png" };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        Ok(format!("data:{};base64,{}", mime, b64))
    } else {
        Err("No icon found in APK".to_string())
    }
}

fn score_icon_path(path: &str, size: u64) -> u32 {
    let mut score = 0u32;
    if path.contains("ic_launcher_foreground") { score += 100; }
    else if path.contains("ic_launcher_round") { score += 80; }
    else if path.contains("ic_launcher") { score += 90; }
    else if path.contains("app_icon") || path.contains("/icon.") { score += 60; }
    else if path.contains("logo") { score += 40; }
    else if path.contains("mipmap") { score += 20; }
    else { return 0; }

    if path.contains("xxxhdpi") { score += 40; }
    else if path.contains("xxhdpi") { score += 30; }
    else if path.contains("xhdpi") { score += 20; }
    else if path.contains("hdpi") { score += 10; }

    if size >= 1_000 && size <= 80_000 { score += 15; }
    else if size >= 80_000 && size <= 200_000 { score += 5; }

    score
}

/// Batch icon load using resolve-app-info (fast, Android 12+).
pub fn get_icons(package_names: Vec<String>) -> HashMap<String, String> {
    let serial = match detect_serial() {
        Some(s) => s,
        None => return HashMap::new(),
    };
    let mut icons = HashMap::new();
    for pkg in &package_names {
        let output = adb_serial(&serial, &["shell", "cmd", "package", "resolve-app-info", "--user", "0", pkg], 8000);
        let line = output.lines().find(|l| l.trim().starts_with("<app-info"));
        if let Some(xml) = line {
            if let Some(start) = xml.find("icon=\"") {
                let rest = &xml[start + 6..];
                if let Some(end) = rest.find('"') {
                    let b64 = &rest[..end];
                    if !b64.is_empty() && b64.len() > 50 {
                        icons.insert(pkg.clone(), b64.to_string());
                    }
                }
            }
        }
    }
    icons
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
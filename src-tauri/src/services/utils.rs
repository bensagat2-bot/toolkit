use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::log;

use serde::Deserialize;
use tauri::{AppHandle, Emitter};

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_resource_dir(path: PathBuf) {
    let _ = RESOURCE_DIR.set(path);
}

// Mirrors v1per-wpf: find a bundled tool folder next to the app by walking up
// the directory tree, falling back to the extracted cache, then to SDK/common
// install paths, then to PATH.
fn find_tool_dir(folder: &str, exe: &str) -> Option<PathBuf> {
    // Portable layout: tools are embedded in the exe and lazily extracted to
    // the per-user versioned cache on first use.
    if let Some(d) = crate::resources::ensure_tool_dir(folder) {
        if d.join(exe).exists() {
            return Some(d);
        }
    }
    if let Some(d) = RESOURCE_DIR.get() {
        let p = d.join(folder);
        if p.join(exe).exists() {
            return Some(p);
        }
    }
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()));
    for _ in 0..8 {
        if let Some(d) = dir {
            // Tauri bundle resources live in a `resources/` subfolder next to the exe.
            let in_resources = d.join("resources").join(folder);
            if in_resources.join(exe).exists() {
                return Some(in_resources);
            }
            let direct = d.join(folder);
            if direct.join(exe).exists() {
                return Some(direct);
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }
    // Android SDK / manual install locations.
    for var in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(home) = std::env::var(var) {
            if !home.is_empty() {
                let p = PathBuf::from(home).join("platform-tools");
                if p.join(exe).exists() {
                    return Some(p);
                }
            }
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let p = PathBuf::from(local).join("Android").join("Sdk").join("platform-tools");
        if p.join(exe).exists() {
            return Some(p);
        }
    }
    if let Some(prog) = std::env::var_os("PROGRAMFILES") {
        let p = PathBuf::from(prog).join("platform-tools");
        if p.join(exe).exists() {
            return Some(p);
        }
    }
    None
}

fn find_tool_exe(folder: &str, exe: &str) -> String {
    find_tool_dir(folder, exe)
        .map(|p| p.join(exe).to_string_lossy().into_owned())
        .unwrap_or_else(|| exe.to_string()) // fallback: PATH
}

pub fn platform_tools_path() -> Option<PathBuf> {
    find_tool_dir("platform-tools", "adb.exe")
}

pub fn adb_path() -> String {
    find_tool_exe("platform-tools", "adb.exe")
}

pub fn fastboot_path() -> String {
    find_tool_exe("platform-tools", "fastboot.exe")
}

pub fn scrcpy_path() -> PathBuf {
    find_tool_dir("scrcpy", "scrcpy.exe").unwrap_or_default()
}

pub fn run_cmd(program: &str, args: &[&str], timeout_ms: u64) -> String {
    run_stdout(base_cmd(program).args(args), timeout_ms)
}

fn emit(app: &AppHandle, msg: &str) {
    let _ = app.emit("utils:progress", serde_json::json!({ "message": msg }));
}

/// Escapes a string for safe use inside a POSIX single-quoted shell string.
fn sh_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    // Wrap in single quotes and replace any embedded single quote
    // with '\'' (close quote, escaped quote, open quote).
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn base_cmd(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

// Runs a command, capturing stdout, and kills the child if it exceeds the
// timeout. The previous blocking read loop never hit its deadline when a child
// (e.g. adb) stalled, and it never piped stdout, so `adb devices` always came
// back empty - which froze the UI on "waiting for device".
pub fn run_output(cmd: &mut Command, timeout_ms: u64) -> std::process::Output {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return default_output(),
    };
    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => return default_output(),
    };
    let mut stderr = match child.stderr.take() {
        Some(s) => s,
        None => return default_output(),
    };
    // fastboot writes getvar values and OKAY/FAILED status to stderr, not
    // stdout. Both streams are piped and merged so callers see the real output.
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let _stdout_reader = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match stdout.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
        let _ = tx.send(buf);
    });
    let (tx2, rx2) = std::sync::mpsc::channel::<Vec<u8>>();
    let _stderr_reader = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match stderr.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
        let _ = tx2.send(buf);
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let mut out_buf = rx.recv().unwrap_or_default();
    let err_buf = rx2.recv().unwrap_or_default();
    if !err_buf.is_empty() {
        out_buf.extend_from_slice(&err_buf);
    }
    let status = child.wait();
    std::process::Output {
        stdout: out_buf,
        stderr: Vec::new(),
        status: status.unwrap_or_default(),
    }
}

fn default_output() -> std::process::Output {
    std::process::Output {
        stdout: Vec::new(),
        stderr: Vec::new(),
        status: std::process::ExitStatus::default(),
    }
}

fn run_stdout(cmd: &mut Command, timeout_ms: u64) -> String {
    String::from_utf8_lossy(&run_output(cmd, timeout_ms).stdout).into_owned()
}

fn adb(args: &[&str], timeout_ms: u64) -> String {
    run_stdout(base_cmd(&adb_path()).args(args), timeout_ms)
}

fn fastboot(args: &[&str], timeout_ms: u64) -> String {
    run_stdout(base_cmd(&fastboot_path()).args(args), timeout_ms)
}

pub(crate) fn adb_serial(serial: &str, args: &[&str], timeout_ms: u64) -> String {
    adb(&["-s", serial].into_iter().chain(args.iter().copied()).collect::<Vec<_>>(), timeout_ms)
}

pub(crate) fn fastboot_serial(serial: &str, args: &[&str], timeout_ms: u64) -> String {
    fastboot(&["-s", serial].into_iter().chain(args.iter().copied()).collect::<Vec<_>>(), timeout_ms)
}

/// Detects the current device mode: "adb", "fastboot", or "none", plus the
/// serial. ADB is checked first since it is the preferred backup path.
pub(crate) fn detect_mode() -> (String, Option<String>) {
    let out = adb(&["devices"], 8000);
    for line in out.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            return ("adb".into(), Some(parts[0].into()));
        }
    }
    let fb = fastboot(&["devices"], 8000);
    for line in fb.lines() {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if !parts.is_empty() && !line.contains("waiting") {
            return ("fastboot".into(), Some(parts[0].into()));
        }
    }
    ("none".into(), None)
}

/// Detects whether a path exists on the device (via su).
pub(crate) fn device_path_exists(serial: &str, path: &str) -> bool {
    let safe = sh_escape(path);
    let out = adb_serial(serial, &["shell", "su", "-c", &format!("test -e {safe} && echo YES")], 8000);
    out.contains("YES")
}

/// Dumps a raw block device (e.g. a partition) from the device to a local file.
/// dd writes to an on-device temp file, then `adb pull` brings it over. This is
/// binary-safe on every su implementation, unlike `adb exec-out su -c dd`
/// which silently produced 0-byte files on several devices. Returns bytes written.
pub(crate) fn dump_partition(serial: &str, src: &str, dest: &std::path::Path) -> Result<u64, String> {
    const REMOTE: &str = "/data/local/tmp/v1per_part.img";
    let cleanup = || {
        let _ = adb_serial(serial, &["shell", "su", "-c", &format!("rm -f {REMOTE}")], 10000);
    };

    cleanup();
    let safe_src = sh_escape(src);
    let dump = adb_serial(
        serial,
        &["shell", "su", "-c", &format!("dd if={safe_src} of={REMOTE} bs=1M 2>/dev/null && sync")],
        600000,
    );
    let listed = adb_serial(serial, &["shell", "ls", "-l", REMOTE], 10000);
    let ok = !listed.to_lowercase().contains("no such")
        && !dump.to_lowercase().contains("denied")
        && !dump.to_lowercase().contains("not found");
    if !ok {
        cleanup();
        return Err(format!("dd failed: {dump}").trim().to_string());
    }

    let mut cmd = base_cmd(&adb_path());
    cmd.args(["-s", serial, "pull", REMOTE, dest.to_string_lossy().as_ref()]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let out = run_output(&mut cmd, 600000);
    // Always remove the temp file from the device so storage is not eaten up.
    cleanup();

    if out.status.success() {
        fs::metadata(dest).map(|m| m.len()).map_err(|e| format!("Failed to stat output: {e}"))
    } else {
        Err(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
}

fn downloads_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| std::env::temp_dir())
}

// User-visible toolkit folder inside Downloads. Holds the backed-up boot
// image and the KernelSU/FolkPatch manager APK the user asked for.
fn toolkit_files_dir() -> PathBuf {
    let d = downloads_dir().join("v1per-toolkit-files");
    std::fs::create_dir_all(&d).ok();
    d
}

// Private scratch folder for helper binaries we push to the phone but must
// not leave behind in the user's Downloads folder.
fn work_tmp_dir() -> PathBuf {
    let d = std::env::temp_dir().join("v1per-toolkit");
    std::fs::create_dir_all(&d).ok();
    d
}

fn fetch_json(url: &str) -> Option<serde_json::Value> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("V1Per-Toolkit/1.0")
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .ok()?;
    client.get(url).send().ok()?.json().ok()
}

fn asset_name(asset: &serde_json::Value) -> String {
    asset.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string()
}

fn asset_url(asset: &serde_json::Value) -> String {
    asset.get("browser_download_url").and_then(|u| u.as_str()).unwrap_or("").to_string()
}

fn pick_apk(release: &serde_json::Value, prefer: &[&str]) -> Option<serde_json::Value> {
    let assets = release.get("assets")?.as_array()?;
    let mut fallback = None;
    for asset in assets {
        let name = asset_name(asset);
        if !name.to_lowercase().ends_with(".apk") {
            continue;
        }
        if name.to_lowercase().contains("spoofed") {
            continue;
        }
        let lower = name.to_lowercase();
        if prefer.iter().any(|p| lower.contains(p)) {
            return Some(asset.clone());
        }
        if fallback.is_none() {
            fallback = Some(asset.clone());
        }
    }
    fallback
}

fn pick_asset(release: &serde_json::Value, exact: &str) -> Option<serde_json::Value> {
    release.get("assets")?.as_array()?.iter().find(|a| asset_name(a) == exact).cloned()
}

fn pick_asset_contains(release: &serde_json::Value, substr: &str) -> Option<serde_json::Value> {
    let lower = substr.to_lowercase();
    release.get("assets")?.as_array()?.iter().find(|a| {
        asset_name(a).to_lowercase().contains(&lower)
    }).cloned()
}

fn download_file_into(_app: &AppHandle, url: &str, filename: &str, dir: &Path) -> Result<PathBuf, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("V1Per-Toolkit/1.0")
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;
    let dest = dir.join(filename);
    if dest.exists() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
        return Ok(dest);
    }
    let resp = client.get(url).send().map_err(|e| format!("HTTP request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} from {}", resp.status(), url));
    }
    let bytes = resp.bytes().map_err(|e| format!("Failed to read response: {e}"))?;
    std::fs::write(&dest, &bytes).map_err(|e| format!("Failed to write file: {e}"))?;
    Ok(dest)
}

fn download_file(app: &AppHandle, url: &str, filename: &str) -> Result<PathBuf, String> {
    let dest = download_file_into(app, url, filename, &downloads_dir())?;
    emit(app, "Downloading... DONE");
    Ok(dest)
}

fn ready_adb_device() -> Result<String, String> {
    let start = std::time::Instant::now();
    let mut last_state = String::new();
    loop {
        let out = adb(&["devices"], 8000);
        let mut seen_device = false;
        for line in out.lines().skip(1) {
            let mut parts = line.split_whitespace();
            let serial = parts.next().map(|s| s.to_string());
            let state = parts.next().unwrap_or("").to_string();
            match state.as_str() {
                "device" => {
                    if let Some(s) = serial {
                        return Ok(s);
                    }
                    seen_device = true;
                }
                "unauthorized" => last_state = "unauthorized".to_string(),
                "offline" => last_state = "offline".to_string(),
                s if !s.is_empty() => {
                    last_state = format!("unknown state ({s})");
                }
                _ => {}
            }
        }
        if seen_device && last_state.is_empty() {
            last_state = "no ready device".to_string();
        }
        if start.elapsed().as_secs() >= 20 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    match last_state.as_str() {
        "unauthorized" => {
            Err("USB debugging is not authorized. Unlock your phone and tap 'Allow' on the RSA debugging prompt.".into())
        }
        "offline" => Err("Device is offline. Reconnect the USB cable and try again.".into()),
        s if !s.is_empty() => Err(format!("No ready ADB device ({s}). Enable USB debugging and connect.")),
        _ => Err("No ADB device detected. Enable USB debugging, connect via USB, and retry.".into()),
    }
}

pub(crate) fn device_prop(serial: &str, prop: &str) -> String {
    adb_serial(serial, &["shell", "getprop", prop], 8000).trim().to_string()
}

/// Ensures shell has permanent SU access without manual phone interaction.
/// 1. Tries `adb root` (restarts adbd as root, no SU prompts ever).
/// 2. Checks if shell already has SU via `su -c id`.
/// 3. If KernelSU detected, auto-grants via `ksud allowlist -add 2000` (user approves ONCE, permanent after).
/// Returns (success, message).
pub fn ensure_su_access() -> (bool, String) {
    // Step 1: Try adb root
    let root_output = run_stdout(base_cmd(&adb_path()).args(["root"]), 8000);
    if root_output.contains("restarting") || root_output.contains("already root") {
        // adb root succeeded or already running as root
        let check = run_stdout(base_cmd(&adb_path()).args(["shell", "id"]), 5000);
        if check.contains("uid=0") {
            log::write("ensure_su_access: adb root active, shell is root");
            return (true, "adb root: shell running as root".into());
        }
        // Give adbd time to restart
        std::thread::sleep(std::time::Duration::from_secs(2));
        let check2 = run_stdout(base_cmd(&adb_path()).args(["shell", "id"]), 5000);
        if check2.contains("uid=0") {
            log::write("ensure_su_access: adb root active after restart");
            return (true, "adb root: shell running as root".into());
        }
    }

    // Step 2: Check if shell already has SU access (user pre-granted)
    let su_check = run_stdout(base_cmd(&adb_path()).args(["shell", "su", "-c", "id"]), 8000);
    if su_check.contains("uid=0") {
        log::write("ensure_su_access: shell already has SU access");
        return (true, "SU access already granted".into());
    }

    // Step 3: Detect KernelSU and auto-grant shell permanently
    let ksu_exists = run_stdout(base_cmd(&adb_path()).args(["shell", "[", "-f", "/data/adb/ksu/bin/ksud", "]", "&&", "echo", "yes"]), 5000);
    if ksu_exists.contains("yes") {
        log::write("ensure_su_access: KernelSU detected, attempting auto-grant via ksud allowlist");
        // Try without su first (some KernelSU builds allow ksud from shell domain)
        let direct = run_stdout(base_cmd(&adb_path()).args(["shell", "/data/adb/ksu/bin/ksud", "allowlist", "-add", "2000"]), 10000);
        if direct.is_empty() || !direct.contains("denied") {
            let verify = run_stdout(base_cmd(&adb_path()).args(["shell", "su", "-c", "id"]), 8000);
            if verify.contains("uid=0") {
                log::write("ensure_su_access: KernelSU auto-grant succeeded via direct ksud");
                return (true, "KernelSU: shell permanently granted SU access".into());
            }
        }
        // Fallback: try via su (user approves ONCE, then permanent)
        let su_allow = run_stdout(base_cmd(&adb_path()).args(["shell", "su", "-c", "/data/adb/ksu/bin/ksud allowlist -add 2000"]), 15000);
        if !su_allow.contains("denied") && !su_allow.contains("error") {
            let verify2 = run_stdout(base_cmd(&adb_path()).args(["shell", "su", "-c", "id"]), 8000);
            if verify2.contains("uid=0") {
                log::write("ensure_su_access: KernelSU auto-grant succeeded via su + ksud");
                return (true, "KernelSU: shell permanently granted SU access".into());
            }
        }
        log::write("ensure_su_access: KernelSU detected but auto-grant failed, needs user action");
        return (false, "KernelSU detected. Open KernelSU → Superuser → tap 'shell' → enable 'Auto-allow'".into());
    }

    // Step 4: Check for APatch
    let apatch_exists = run_stdout(base_cmd(&adb_path()).args(["shell", "[", "-f", "/data/adb/apd", "]", "&&", "echo", "yes"]), 5000);
    if apatch_exists.contains("yes") {
        log::write("ensure_su_access: APatch detected, attempting auto-grant via apd");
        let apd_out = run_stdout(base_cmd(&adb_path()).args(["shell", "su", "-c", "/data/adb/apd allow -add 2000"]), 10000);
        if !apd_out.contains("denied") && !apd_out.contains("error") {
            let verify = run_stdout(base_cmd(&adb_path()).args(["shell", "su", "-c", "id"]), 8000);
            if verify.contains("uid=0") {
                log::write("ensure_su_access: APatch auto-grant succeeded");
                return (true, "APatch: shell permanently granted SU access".into());
            }
        }
        return (false, "APatch detected. Open APatch → Superuser → tap 'shell' → enable 'Auto-allow'".into());
    }

    // Step 5: Check for Magisk
    let magisk_exists = run_stdout(base_cmd(&adb_path()).args(["shell", "[", "-f", "/data/adb/magisk", "]", "&&", "echo", "yes"]), 5000)
        .trim().to_string();
    let magisk_db = run_stdout(base_cmd(&adb_path()).args(["shell", "[", "-f", "/data/adb/magisk.db", "]", "&&", "echo", "yes"]), 5000)
        .trim().to_string();
    if magisk_exists.contains("yes") || magisk_db.contains("yes") {
        log::write("ensure_su_access: Magisk detected, cannot auto-grant");
        return (false, "Magisk detected. Open Magisk → Superuser → tap 'shell' → Grant".into());
    }

    log::write("ensure_su_access: no supported root manager detected");
    (false, "No supported root manager detected. Ensure the device is rooted.".into())
}

fn fastboot_devices() -> Vec<String> {
    let out = fastboot(&["devices"], 8000);
    let mut result = Vec::new();
    for line in out.lines() {
        let t = line.trim();
        if t.is_empty() || t.contains("waiting") {
            continue;
        }
        if t.contains("no permission") {
            continue;
        }
        let parts: Vec<&str> = t.split_whitespace().collect();
        if !parts.is_empty() {
            result.push(parts[0].to_string());
        }
    }
    result
}

/// Checks if fastboot is operational and detects common driver issues.
pub fn check_fastboot_driver() -> (bool, String) {
    let exe = fastboot_path();
    if !std::path::Path::new(&exe).exists() && exe == "fastboot.exe" {
        return (false, "fastboot: bundled platform-tools missing (reinstall the toolkit)".into());
    }
    let out = run_stdout(base_cmd(&exe).args(["devices"]), 8000);
    if out.contains("no permission") {
        (false, "fastboot: device detected but no permission. Re-run as administrator or install fastboot driver.".into())
    } else if out.trim().is_empty() || out.contains("waiting") {
        (false, "fastboot: no devices found. Ensure the device is in bootloader mode and the fastboot driver is installed.".into())
    } else {
        (true, String::new())
    }
}

// Reads `fastboot getvar unlocked` live and returns whether the value is "yes".
// Xiaomi returns "(bootloader) unlocked: yes" when unlocked and ": no" when
// locked. If the device is not ready yet the command returns empty, so we
// retry a few times before deciding.
fn is_unlocked(serial: &str) -> bool {
    for _ in 0..4 {
        let out = fastboot_serial(serial, &["getvar", "unlocked"], 15000);
        let lower = out.to_lowercase();
        if let Some(val) = lower.split("unlocked:").nth(1) {
            let v = val.trim_start();
            if v.starts_with('y') {
                return true;
            }
            if v.starts_with('n') {
                return false;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
    // Fallback: some devices only expose the flag via `getvar all`.
    let all = fastboot_serial(serial, &["getvar", "all"], 15000);
    all.to_lowercase().contains("unlocked: yes")
}

fn flash_looks_ok(output: &str) -> bool {
    let lower = output.to_lowercase();
    if lower.contains("failed") || lower.contains("error:") || lower.contains("unknown partition") {
        return false;
    }
    lower.contains("okay") || lower.contains("finished") || lower.contains("sending")
}

// ── Driver download ─────────────────────────────────────────

/// Downloads a driver zip from a releases URL to the Downloads folder.
pub fn driver_download(app: AppHandle, name: String, url: String) -> Result<bool, String> {
    emit(&app, format!("Downloading [{}] from releases...", name).as_str());
    let filename = url.rsplit('/').next().filter(|f| f.contains('.')).unwrap_or("driver.zip");
    let dest = download_file(&app, &url, filename)?;
    emit(&app, format!("Saving to Downloads folder... DONE ({})", dest.display()).as_str());
    emit(&app, "Enjoy!!!");
    Ok(true)
}

// ── Root ─────────────────────────────────────────────────────

const FOLKPATCH_BANNER: &str = include_str!("../../banner-folkpatch.txt");
const KERNELSU_BANNER: &str = include_str!("../../banner.txt");
const KERNELSU_NEXT_BANNER: &str = include_str!("../../banner-kernelsu-next.txt");
const SUKISU_BANNER: &str = include_str!("../../banner-sukisu.txt");

#[derive(Debug, Clone, Deserialize)]
pub struct RootOptions {
    pub boot_img: String,
    pub manager: Option<String>,
}

/// Runs the KernelSU-Next / KernelSU / SukiSU-Ultra / FolkPatch root pipeline
/// using the manager selected in the UI dropdown. The chosen manager's ASCII
/// banner is printed first so the user knows which root manager will be installed.
pub fn root(app: AppHandle, opts: RootOptions) -> Result<bool, String> {
    let started = std::time::Instant::now();
    let boot_img = Path::new(&opts.boot_img);
    if !boot_img.exists() {
        return Err(format!("File not found: {}", opts.boot_img));
    }

    // Manager chosen by the user in the dropdown; defaults to KernelSU-Next.
    let mgr = opts.manager.as_deref().unwrap_or("ksu-next");
    let (release_url, manager_label, apk_label, prefer) = match mgr {
        "folkpatch" => {
            emit(&app, FOLKPATCH_BANNER.trim_end());
            ("https://api.github.com/repos/LyraVoid/FolkPatch/releases/latest", "FolkPatch", "FolkPatch.apk", "folkpatch")
        }
        "ksu" => {
            emit(&app, KERNELSU_BANNER.trim_end());
            ("https://api.github.com/repos/tiann/KernelSU/releases/latest", "KernelSU", "KernelSU_v", "release")
        }
        "sukisu" => {
            emit(&app, SUKISU_BANNER.trim_end());
            ("https://api.github.com/repos/SukiSU-Ultra/SukiSU-Ultra/releases/latest", "SukiSU-Ultra", "SukiSU_v", "release")
        }
        _ => {
            emit(&app, KERNELSU_NEXT_BANNER.trim_end());
            ("https://api.github.com/repos/KernelSU-Next/KernelSU-Next/releases/latest", "KernelSU-Next", "KernelSU_Next", "spoofed")
        }
    };
    emit(&app, "Checking ADB Connection...");
    let serial = match ready_adb_device() {
        Ok(s) => {
            emit(&app, "Checking ADB Connection... FOUND");
            s
        }
        Err(e) => {
            emit(&app, "Checking ADB Connection... NOT FOUND");
            return Err(e);
        }
    };

    // Auto-grant SU access for shell so user does not need to tap phone.
    let (_su_ok, _su_msg) = ensure_su_access();

    let work = work_tmp_dir();
    let files_dir = toolkit_files_dir();

    let backup_name = format!(
        "backup-{}",
        boot_img
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "boot.img".to_string())
    );
    let backup_path = files_dir.join(&backup_name);
    std::fs::copy(boot_img, &backup_path)
        .map_err(|e| format!("Failed to back up boot image: {e}"))?;
    emit(&app, format!("Backed up boot image -> {}", backup_path.display()).as_str());

    let model = device_prop(&serial, "ro.product.model");
    let sdk = device_prop(&serial, "ro.build.version.sdk");
    let release = device_prop(&serial, "ro.build.version.release");
    let platform = device_prop(&serial, "ro.board.platform");
    let abi = device_prop(&serial, "ro.product.cpu.abi");
    emit(&app, format!("ro.product.model : {}", model).as_str());
    emit(&app, format!("ro.build.version.sdk : {}", sdk).as_str());
    emit(&app, format!("ro.build.version.release : {}", release).as_str());
    emit(&app, format!("ro.board.platform : {}", platform).as_str());

    // Xiaomi bootloader lock check: if Xiaomi/Mi/Redmi/POCO and bootloader
    // is still locked, abort early with a clear message.
    let lower_model = model.to_lowercase();
    let is_xiaomi_brand = lower_model.contains("xiaomi")
        || lower_model.contains("mi ")
        || lower_model.starts_with("mi ")
        || lower_model.starts_with("mi")
        || lower_model.contains("redmi")
        || lower_model.contains("poco");
    if is_xiaomi_brand {
        emit(&app, "Xiaomi device detected, checking bootloader status...");
        let flash_locked = device_prop(&serial, "ro.boot.flash.locked");
        let verified = device_prop(&serial, "ro.boot.verifiedbootstate");
        if flash_locked.trim() == "1" || verified.trim() == "green" {
            emit(&app, "Xiaomi device detected, checking bootloader status... LOCKED");
            return Err(
                "Bootloader is not unlocked yet. Please unlock it manually first.\n\
                 Xiaomi requires official bootloader unlock via Mi Unlock tool.\n\
                 This tool cannot unlock Xiaomi bootloaders. Unlock manually, then retry.".into()
            );
        }
        emit(&app, "Xiaomi device detected, checking bootloader status... UNLOCKED");
    }

    let release = fetch_json(release_url).ok_or("Failed to fetch release. Check internet.")?;
    let apk = pick_apk(&release, &[prefer]).ok_or("Failed to fetch release asset.")?;
    let apk_url = asset_url(&apk);
    let apk_name = asset_name(&apk);

    emit(&app, format!("Downloading [{}]...", apk_label).as_str());
    let apk_path = download_file_into(&app, &apk_url, &apk_name, &files_dir)?;
    emit(&app, format!("Downloading [{}]... DONE", apk_label).as_str());

    emit(&app, format!("Installing [{}] on device...", manager_label).as_str());
    let install = adb_serial(&serial, &["install", "-r", apk_path.to_string_lossy().as_ref()], 120000);
    if !install.to_lowercase().contains("success") {
        return Err(format!("Install failed: {}", install));
    }
    emit(&app, format!("Installing [{}] on device... DONE", manager_label).as_str());

    let part_name = partition_from_image(boot_img);

    // Only ksud-based managers can handle init_boot. FolkPatch needs boot.img.
    if mgr == "folkpatch" {
        let n = boot_img.file_name().map(|f| f.to_string_lossy().to_lowercase()).unwrap_or_default();
        if n.contains("init_boot") {
            return Err(format!("{} needs boot.img, not init_boot.img.", manager_label));
        }
    }

    emit(&app, "Patching boot image...");
    let patched_local = match mgr {
        "folkpatch" => auto_patch_folkpatch(&app, &serial, &work, boot_img.to_string_lossy().as_ref())?,
        "sukisu" => {
            let sukisu_ksud = extract_ksud_from_apk(&apk_path, &work)?;
            auto_patch_ksud(&app, &serial, &work, boot_img.to_string_lossy().as_ref(), &release, &abi, Some(sukisu_ksud))?
        },
        _ => auto_patch_ksud(&app, &serial, &work, boot_img.to_string_lossy().as_ref(), &release, &abi, None)?,
    };
    let Some(patched_local) = patched_local else {
        return Err("Auto-patch failed.".into());
    };
    emit(&app, format!("Patching complete: {}", patched_local).as_str());

    // Keep the patched image for the user: a copy in the v1per-toolkit-files
    // folder and one directly in Downloads, so it can be re-flashed later via
    // fastboot without running the whole patch flow again.
    let patched_name = format!("patched_{}.img", part_name);
    let patched_saved = files_dir.join(&patched_name);
    std::fs::copy(&patched_local, &patched_saved)
        .map_err(|e| format!("Failed to save patched image: {e}"))?;
    std::fs::copy(&patched_local, downloads_dir().join(&patched_name)).ok();
    emit(&app, format!("Patched image saved -> {}", patched_saved.display()).as_str());

    // Remove the helper binaries and images we pushed to the device. The
    // patched image is already pulled to the PC, so nothing on the phone is
    // needed for the fastboot flash.
    emit(&app, "Cleaning up...");
    let remote_junk = [
        "/data/local/tmp/ksud",
        "/data/local/tmp/kptools",
        "/data/local/tmp/kpimg-android",
        "/data/local/tmp/kpimg",
        "/data/local/tmp/v1per_kp",
        "/data/local/tmp/v1per_input.img",
        "/data/local/tmp/v1per_part.img",
        "/sdcard/Download/v1per_input.img",
        "/sdcard/Download/kernelsu_patched.img",
        "/sdcard/Download/folkpatch_patched.img",
        "/sdcard/Download/patched_sukisu.img",
    ];
    for path in remote_junk {
        let _ = adb_serial(&serial, &["shell", "rm", "-rf", path], 10000);
    }
    emit(&app, "Cleaning up... DONE");

    flash_patched(&app, &serial, part_name, patched_saved.to_string_lossy().as_ref())?;

    // Scratch copy in the temp dir is no longer needed; the Downloads copies stay.
    let _ = std::fs::remove_file(Path::new(&patched_local));

    let elapsed = started.elapsed().as_secs();
    emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
    Ok(true)
}

fn partition_from_image(path: &Path) -> &'static str {
    let name = path.file_name().map(|f| f.to_string_lossy().to_lowercase()).unwrap_or_default();
    if name.contains("init_boot") {
        "init_boot"
    } else if name.contains("vendor_boot") {
        "vendor_boot"
    } else {
        "boot"
    }
}

/// Extracts a KMI string like "android14-6.1" from a kernel string.
fn parse_kmi(text: &str) -> Option<String> {
    let lower = text.replace('_', "-").to_lowercase();
    let bytes = lower.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    while i < n {
        if i + 7 <= n && &lower[i..i + 7] == "android" {
            let mut j = i + 7;
            let digits_start = j;
            while j < n && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let digits = &lower[digits_start..j];
            if !digits.is_empty() && j < n && bytes[j] == b'-' {
                j += 1;
                let ver_start = j;
                while j < n && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                    j += 1;
                }
                let ver = &lower[ver_start..j];
                if ver.contains('.') {
                    return Some(format!("android{digits}-{ver}"));
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

fn detect_kmi(serial: &str, remote_ksud: Option<&str>) -> Option<String> {
    if let Some(ksud) = remote_ksud {
        let out0 = adb_serial(serial, &["shell", ksud, "boot-info", "current-kmi"], 20000);
        if let Some(kmi) = parse_kmi(&out0) {
            return Some(kmi);
        }
    }
    let out = adb_serial(serial, &["shell", "uname", "-r"], 8000);
    if let Some(kmi) = parse_kmi(&out) {
        return Some(kmi);
    }
    for prop in ["ro.kernel.version", "ro.boot.kernel", "ro.build.version.release"] {
        let p = device_prop(serial, prop);
        if let Some(kmi) = parse_kmi(&p) {
            return Some(kmi);
        }
    }
    let ver = adb_serial(serial, &["shell", "cat", "/proc/version"], 8000);
    parse_kmi(&ver)
}

fn pick_ksud(release: &serde_json::Value, abi: &str) -> Option<serde_json::Value> {
    let want = match abi {
        "x86_64" => "ksud-x86_64-linux-android",
        "armeabi-v7a" | "armeabi" => "ksud-armv7-linux-androideabi",
        _ => "ksud-aarch64-linux-android",
    };
    if let Some(a) = pick_asset(release, want) {
        return Some(a);
    }
    release.get("assets")?.as_array()?.iter().find(|a| {
        let n = asset_name(a);
        n.starts_with("ksud-") && n.contains("android")
    }).cloned()
}

/// Extracts `lib/arm64-v8a/libksud.so` from a SukiSU APK (which is a zip)
/// and saves it as `ksud` in the work directory.
fn extract_ksud_from_apk(apk_path: &Path, work: &Path) -> Result<PathBuf, String> {
    use std::io::Read;
    let file = std::fs::File::open(apk_path)
        .map_err(|e| format!("Failed to open APK: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read APK as zip: {e}"))?;
    let mut found = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {e}"))?;
        let name = entry.name().to_lowercase().replace('\\', "/");
        if name == "lib/arm64-v8a/libksud.so" {
            let dest = work.join("ksud");
            let mut out = std::fs::File::create(&dest)
                .map_err(|e| format!("Failed to create ksud file: {e}"))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| format!("Failed to extract ksud: {e}"))?;
            found = Some(dest);
            break;
        }
    }
    found.ok_or_else(|| "lib/arm64-v8a/libksud.so not found in APK".into())
}
fn auto_patch_ksud(
    app: &AppHandle,
    serial: &str,
    work: &Path,
    image_path: &str,
    release: &serde_json::Value,
    abi: &str,
) -> Result<Option<String>, String> {
    let ksud = pick_ksud(release, abi).ok_or("No on-device ksud in this release.")?;
    let ksud_local = download_file_into(app, &asset_url(&ksud), &asset_name(&ksud), work)?;

    const REMOTE_KSUD: &str = "/data/local/tmp/ksud";
    const REMOTE_IN: &str = "/sdcard/Download/v1per_input.img";
    const REMOTE_OUT_NAME: &str = "kernelsu_patched.img";
    let remote_out = format!("/sdcard/Download/{REMOTE_OUT_NAME}");

    adb_serial(serial, &["shell", "mkdir", "-p", "/sdcard/Download"], 10000);
    adb_serial(serial, &["push", ksud_local.to_string_lossy().as_ref(), REMOTE_KSUD], 60000);
    adb_serial(serial, &["push", image_path, REMOTE_IN], 120000);
    adb_serial(serial, &["shell", "chmod", "755", REMOTE_KSUD], 10000);
    adb_serial(serial, &["shell", "rm", "-f", remote_out.as_str()], 10000);

    // The ksud helper lives on the device now; never leave it in the user's folder.
    let _ = std::fs::remove_file(&ksud_local);

    emit(app, "Reading KMI...");
    let kmi = detect_kmi(serial, Some(REMOTE_KSUD));
    if let Some(kmi) = &kmi {
        emit(app, format!("KMI: {kmi}").as_str());
    } else {
        emit(app, "No KMI string on this kernel. Letting ksud read it from the image.");
    }

    let mut patch_args = vec![
        "shell".to_string(),
        REMOTE_KSUD.to_string(),
        "boot-patch".to_string(),
        "-b".to_string(),
        REMOTE_IN.to_string(),
        "-o".to_string(),
        "/sdcard/Download".to_string(),
        "--out-name".to_string(),
        REMOTE_OUT_NAME.to_string(),
    ];
    if let Some(kmi) = kmi {
        patch_args.push("--kmi".to_string());
        patch_args.push(kmi);
    }
    let refs: Vec<&str> = patch_args.iter().map(|s| s.as_str()).collect();
    emit(app, "Waiting for patched image...");
    adb_serial(serial, &refs, 180000);

    let exists = adb_serial(serial, &["shell", "ls", remote_out.as_str()], 10000);
    if exists.trim().is_empty() || exists.to_lowercase().contains("no such") {
        return Err("ksud did not produce a patched image.".into());
    }

    let patched_local = work.join("patched_boot.img");
    adb_serial(serial, &["pull", remote_out.as_str(), patched_local.to_string_lossy().as_ref()], 120000);
    let ok = std::fs::metadata(&patched_local).map(|m| m.len() >= 4096).unwrap_or(false);
    if !ok {
        return Err("Failed to pull patched image.".into());
    }
    emit(app, "Patching boot image... DONE");
    Ok(Some(patched_local.to_string_lossy().into_owned()))
}

fn auto_patch_folkpatch(app: &AppHandle, serial: &str, work: &Path, image_path: &str) -> Result<Option<String>, String> {
    let release = fetch_json("https://api.github.com/repos/bmax121/KernelPatch/releases/latest")
        .ok_or("Failed to fetch KernelPatch release. Check internet.")?;
    let tools = pick_asset(&release, "kptools-android").ok_or("No kptools-android in KernelPatch release.")?;
    let kpimg = pick_asset(&release, "kpimg-android").ok_or("No kpimg-android in KernelPatch release.")?;

    let tools_local = download_file_into(app, &asset_url(&tools), &asset_name(&tools), work)?;
    let kpimg_local = download_file_into(app, &asset_url(&kpimg), &asset_name(&kpimg), work)?;

    const REMOTE_BIN: &str = "/data/local/tmp/kptools";
    const REMOTE_KPIMG: &str = "/data/local/tmp/kpimg-android";
    const WORK: &str = "/data/local/tmp/v1per_kp";
    let remote_in = format!("{WORK}/boot.img");
    let remote_out = "/sdcard/Download/folkpatch_patched.img".to_string();

    adb_serial(serial, &["shell", "rm", "-rf", WORK], 10000);
    adb_serial(serial, &["shell", "mkdir", "-p", WORK], 10000);
    adb_serial(serial, &["shell", "mkdir", "-p", "/sdcard/Download"], 10000);
    adb_serial(serial, &["push", tools_local.to_string_lossy().as_ref(), REMOTE_BIN], 60000);
    adb_serial(serial, &["push", kpimg_local.to_string_lossy().as_ref(), REMOTE_KPIMG], 60000);
    adb_serial(serial, &["push", image_path, remote_in.as_str()], 120000);
    adb_serial(serial, &["shell", "chmod", "755", REMOTE_BIN], 10000);
    adb_serial(serial, &["shell", "rm", "-f", remote_out.as_str()], 10000);

    // Direct single-step patch: kptools -p detects ANDROID! magic, extracts kernel,
    // patches it with kpimg, and repacks automatically.
    emit(app, "Waiting for patched image...");
    adb_serial(serial, &[
        "shell", REMOTE_BIN, "-p", "--image", remote_in.as_str(), "--skey", "su",
        "--kpimg", REMOTE_KPIMG, "--out", remote_out.as_str(),
    ], 180000);

    let exists = adb_serial(serial, &["shell", "ls", remote_out.as_str()], 10000);
    if exists.trim().is_empty() || exists.to_lowercase().contains("no such") {
        return Err("kptools did not produce a patched image.".into());
    }

    let patched_local = work.join("patched_boot.img");
    adb_serial(serial, &["pull", remote_out.as_str(), patched_local.to_string_lossy().as_ref()], 120000);
    let ok = std::fs::metadata(&patched_local).map(|m| m.len() >= 4096).unwrap_or(false);
    if !ok {
        return Err("Failed to pull patched image.".into());
    }
    emit(app, "Patching boot image... DONE");
    Ok(Some(patched_local.to_string_lossy().into_owned()))
}
fn flash_patched(app: &AppHandle, serial: &str, partition: &str, patched_local: &str) -> Result<(), String> {
    emit(app, "Rebooting device to bootloader mode...");
    adb_serial(serial, &["reboot", "bootloader"], 15000);
    emit(app, "Rebooting device to bootloader mode... DONE");

emit(app, "Waiting for device in fastboot mode...");
    let mut fb_serial = None;
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let devs = fastboot_devices();
        if let Some(d) = devs.first() {
            fb_serial = Some(d.clone());
            break;
        }
    }
let mut fb_serial = match fb_serial {
        Some(d) => {
            emit(&app, "Waiting for device in fastboot mode... FOUND");
            d
        }
        None => {
            emit(&app, "Waiting for device in fastboot mode... NOT FOUND");
            let (ok, msg) = check_fastboot_driver();
            if !ok {
                emit(&app, &msg);
                return Err(format!("Fastboot device not detected. {msg}"));
            }
            return Err("Fastboot device not detected after reboot.".into());
        }
    };

    emit(app, "Checking bootloader status...");
    let mut unlocked = is_unlocked(&fb_serial);
    if unlocked {
        emit(app, "Checking bootloader status... UNLOCKED");
    } else {
        emit(app, "Checking bootloader status... LOCKED");
        emit(app, "Attempting to unlock bootloader...");
        for attempt in 1..=5 {
            emit(app, format!("Attempt {attempt}/5").as_str());
            emit(app, "Please confirm unlock on your device (press VOLUME UP when prompted)");
            let r = fastboot_serial(&fb_serial, &["flashing", "unlock"], 20000);
            if r.trim().is_empty() || r.to_lowercase().contains("unknown") {
                fastboot_serial(&fb_serial, &["oem", "unlock"], 20000);
            }
            std::thread::sleep(std::time::Duration::from_secs(4));
            if let Some(d) = fastboot_devices().first() {
                fb_serial = d.clone();
            }
            if is_unlocked(&fb_serial) {
                emit(app, "Attempting to unlock bootloader... DONE");
                emit(app, "Bootloader successfully unlocked");
                unlocked = true;
                break;
            }
            if attempt < 5 {
                emit(app, "Unlock attempt failed, retrying...");
            }
        }
        if !unlocked {
            emit(app, "Could not verify unlock status. Continuing...");
        }
    }

    emit(app, format!("Flashing partition \"{partition}\"...").as_str());
    let mut ok = flash_looks_ok(&fastboot_serial(&fb_serial, &["flash", partition, patched_local], 90000));
    if !ok {
        for slot in ["a", "b"] {
            let r = fastboot_serial(&fb_serial, &["flash", format!("{partition}_{slot}").as_str(), patched_local], 90000);
            if flash_looks_ok(&r) {
                ok = true;
                break;
            }
        }
    }
    if ok {
        emit(app, "Flash successful, rebooting device...");
        fastboot_serial(&fb_serial, &["reboot"], 10000);
    } else {
        return Err("Flash may have failed.".into());
    }
    Ok(())
}

fn auto_patch_kptools(
    app: &AppHandle,
    serial: &str,
    work: &Path,
    image_path: &str,
    release_url: &str,
    kpimg_name: &str,
) -> Result<Option<String>, String> {
    // Download kptools from KernelPatch (has unpack/repack, needed for -p auto mode)
    let kp_release = fetch_json("https://api.github.com/repos/bmax121/KernelPatch/releases/latest")
        .ok_or("Failed to fetch KernelPatch release. Check internet.")?;
    let kp_tools = pick_asset(&kp_release, "kptools-android")
        .ok_or("No kptools-android in KernelPatch release.")?;
    let tools_local = download_file_into(app, &asset_url(&kp_tools), &asset_name(&kp_tools), work)?;

    // Download SukiSU's kpimg
    let sk_release = fetch_json(release_url)
        .ok_or("Failed to fetch SukiSU KernelPatch release. Check internet.")?;
    let kpimg = pick_asset_contains(&sk_release, kpimg_name)
        .ok_or_else(|| {
            let names: Vec<String> = sk_release.get("assets")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().map(asset_name).collect())
                .unwrap_or_default();
            format!("No kpimg asset in SukiSU release. Found: {:?}", names)
        })?;
    let kpimg_local = download_file_into(app, &asset_url(&kpimg), &asset_name(&kpimg), work)?;

    const REMOTE_BIN: &str = "/data/local/tmp/kptools";
    const REMOTE_KPIMG: &str = "/data/local/tmp/kpimg";
    const WORK: &str = "/data/local/tmp/v1per_kp";
    let remote_in = format!("{WORK}/boot.img");
    let remote_out = "/sdcard/Download/patched_sukisu.img".to_string();

    adb_serial(serial, &["shell", "rm", "-rf", WORK], 10000);
    adb_serial(serial, &["shell", "mkdir", "-p", WORK], 10000);
    adb_serial(serial, &["push", tools_local.to_string_lossy().as_ref(), REMOTE_BIN], 60000);
    adb_serial(serial, &["push", kpimg_local.to_string_lossy().as_ref(), REMOTE_KPIMG], 60000);
    adb_serial(serial, &["push", image_path, remote_in.as_str()], 120000);
    adb_serial(serial, &["shell", "chmod", "755", REMOTE_BIN], 10000);
    adb_serial(serial, &["shell", "rm", "-f", remote_out.as_str()], 10000);
    let _ = std::fs::remove_file(&tools_local);
    let _ = std::fs::remove_file(&kpimg_local);

    // Direct single-step patch: kptools -p detects ANDROID! magic, extracts kernel,
    // patches it with SukiSU's kpimg, and repacks automatically.
    emit(app, "Waiting for patched image...");
    adb_serial(serial, &[
        "shell", REMOTE_BIN, "-p", "--image", remote_in.as_str(), "--skey", "su",
        "--kpimg", REMOTE_KPIMG, "--out", remote_out.as_str(),
    ], 180000);

    let exists = adb_serial(serial, &["shell", "ls", remote_out.as_str()], 10000);
    if exists.trim().is_empty() || exists.to_lowercase().contains("no such") {
        return Err("kptools did not produce a patched image.".into());
    }

    let patched_local = work.join("patched_boot.img");
    adb_serial(serial, &["pull", remote_out.as_str(), patched_local.to_string_lossy().as_ref()], 120000);
    let ok = std::fs::metadata(&patched_local).map(|m| m.len() >= 4096).unwrap_or(false);
    if !ok {
        return Err("Failed to pull patched image.".into());
    }
    emit(app, "Patching boot image... DONE");
    Ok(Some(patched_local.to_string_lossy().into_owned()))
}
// ── Scrcpy ──────────────────────────────────────────────────

fn scrcpy_running() -> bool {
    let out = run_stdout(base_cmd("tasklist").args(["/FI", "IMAGENAME eq scrcpy.exe"]), 5000);
    out.to_lowercase().contains("scrcpy.exe")
}

/// Launches the embedded scrcpy mirroring session via scrcpy-noconsole.vbs.
pub fn launch_scrcpy(app: AppHandle) -> Result<bool, String> {
    let dir = scrcpy_path();
    let vbs = dir.join("scrcpy-noconsole.vbs");
    if !vbs.exists() {
        return Err("scrcpy files not found. Reinstall the toolkit to restore embedded files.".into());
    }

    emit(&app, "Checking ADB connection...");
    let serial = match ready_adb_device() {
        Ok(s) => {
            emit(&app, "Checking ADB connection... FOUND");
            s
        }
        Err(e) => {
            emit(&app, "Checking ADB connection... NOT FOUND");
            return Err(e);
        }
    };
    emit(&app, format!("Device: {}", serial).as_str());

    if scrcpy_running() {
        emit(&app, "scrcpy is already running.");
        emit(&app, "OK");
        return Ok(true);
    }

    emit(&app, "Running scrcpy-noconsole.vbs please wait....");
    let mut cmd = base_cmd("wscript.exe");
    cmd.current_dir(&dir).arg("scrcpy-noconsole.vbs");
    // Put the bundled platform-tools on PATH so scrcpy can find adb.
    if let Some(tools) = platform_tools_path() {
        let existing = std::env::var("PATH").unwrap_or_default();
        cmd.env("PATH", format!("{};{}", tools.to_string_lossy(), existing));
    }
    let child = cmd.spawn().map_err(|e| format!("Failed to launch scrcpy: {e}"))?;
    drop(child);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut launched = false;
    while std::time::Instant::now() < deadline {
        if scrcpy_running() {
            launched = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    if launched {
        emit(&app, "OK");
        Ok(true)
    } else {
        Err("scrcpy failed to start. Make sure the phone screen is unlocked and USB debugging is on.".into())
    }
}

// ── Force Fastboot ───────────────────────────────────────────

const MTK_VENDOR_ID: u16 = 0x0E8D;
const BOOT_MODE_CMD: &[u8] = b"FASTBOOT";
const MTK_KEYWORDS: [&str; 3] = ["mediatek", "preloader", "mtk"];

/// Floods the MTK preloader serial port with "FASTBOOT" to force the device into fastboot mode.
/// Mirrors penumbra-wrapper fastboot.rs: ranks ports by VID 0e8d + USB descriptor
/// keywords, then floods and probes each candidate for the preloader ACK.
pub fn force_fastboot(app: AppHandle) -> Result<bool, String> {
    let started = std::time::Instant::now();
    emit(&app, "Forcing it now.....");
    emit(&app, "Attempting to switch device to Fastboot...");

    if !fastboot_devices().is_empty() {
        emit(&app, "Device already in Fastboot mode.");
        emit(&app, "OK DONE");
        return Ok(true);
    }

    let expected_ack = {
        let mut ack = b"READY".to_vec();
        let tail: Vec<u8> = BOOT_MODE_CMD.iter().rev().take(3).copied().collect();
        ack.extend(tail);
        ack
    };

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let mut saw_device = false;
    let mut saw_no_ack = false;
    let mut saw_open_error = false;
    let mut last_attempt: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();
    let mut last_mode_check = std::time::Instant::now();

    while std::time::Instant::now() < deadline {
        if last_mode_check.elapsed() >= std::time::Duration::from_millis(500) {
            if !fastboot_devices().is_empty() {
                emit(&app, "Device switched to Fastboot during handshake.");
                emit(&app, "OK DONE");
                return Ok(true);
            }
            last_mode_check = std::time::Instant::now();
        }

        let ports = match serialport::available_ports() {
            Ok(p) => p,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(200));
                continue;
            }
        };
        let candidates = rank_ports(ports, MTK_VENDOR_ID, &MTK_KEYWORDS);
        if candidates.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(200));
            continue;
        }

        for port in &candidates {
            let now = std::time::Instant::now();
            if last_attempt.get(&port.port_name).map(|t| (now - *t).as_millis() < 500).unwrap_or(false) {
                continue;
            }
            last_attempt.insert(port.port_name.clone(), now);
            saw_device = true;
            emit(&app, format!("Detected candidate MTK port: {}", port.port_name).as_str());

            for baud in [115200u32, 921600, 57600] {
                match attempt_fastboot(&port.port_name, baud, &expected_ack) {
                    AttemptResult::Success => {
                        emit(&app, "Handshake successful. Device should be in Fastboot.");
                        emit(&app, "OK DONE");
                        let elapsed = started.elapsed().as_secs();
                        emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
                        return Ok(true);
                    }
                    AttemptResult::OpenError(_) => saw_open_error = true,
                    AttemptResult::NoAck => saw_no_ack = true,
                    AttemptResult::Disconnected => {
                        emit(&app, format!("Port {} dropped mid-handshake; device may be switching to Fastboot.", port.port_name).as_str());
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    if !fastboot_devices().is_empty() {
        emit(&app, "Device switched to Fastboot during handshake.");
        emit(&app, "OK DONE");
        return Ok(true);
    }
    if !saw_device {
        emit(&app, "No MTK preloader device detected.");
        return Err("Make sure the phone is powered off and connected via USB.".into());
    }
    if saw_open_error && !saw_no_ack {
        return Err("Ports detected but could not be opened. Check driver permissions for VID 0e8d.".into());
    }
    Err("Device detected but no ACK received from preloader.".into())
}

enum AttemptResult {
    Success,
    #[allow(dead_code)]
    OpenError(String),
    NoAck,
    Disconnected,
}

/// Scores ports by USB VID (0e8d = MediaTek) plus product/manufacturer keywords,
/// so "COM5" style names still match on Windows. Returns highest scored first.
fn rank_ports(ports: Vec<serialport::SerialPortInfo>, vid: u16, keywords: &[&str]) -> Vec<serialport::SerialPortInfo> {
    let mut candidates: Vec<(i32, serialport::SerialPortInfo)> = Vec::new();
    for port in ports {
        let mut score = 0;
        let mut desc = String::new();
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            if info.vid == vid {
                score += 100;
            }
            if let Some(p) = &info.product {
                desc.push_str(p);
                desc.push(' ');
            }
            if let Some(m) = &info.manufacturer {
                desc.push_str(m);
                desc.push(' ');
            }
        }
        let lower_desc = desc.to_lowercase();
        for keyword in keywords {
            if lower_desc.contains(keyword) {
                score += 10;
            }
        }
        if score > 0 {
            candidates.push((score, port));
        }
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.port_name.cmp(&b.1.port_name)));
    candidates.into_iter().map(|(_, p)| p).collect()
}

/// Floods "FASTBOOT" for ~1.5s then probes for the ACK, returning the outcome.
fn attempt_fastboot(port_name: &str, baud: u32, expected_ack: &[u8]) -> AttemptResult {
    let mut port: Box<dyn serialport::SerialPort> = match serialport::new(port_name, baud)
        .timeout(std::time::Duration::from_millis(1))
        .open()
    {
        Ok(p) => p,
        Err(e) => return AttemptResult::OpenError(e.to_string()),
    };

    let start = std::time::Instant::now();
    let flood_duration = std::time::Duration::from_millis(1500);
    let probe_duration = std::time::Duration::from_millis(1000);
    let mut last_mode_check = std::time::Instant::now();

    while std::time::Instant::now().duration_since(start) < flood_duration {
        if port.write(BOOT_MODE_CMD).is_err() {
            return if in_fastboot_mode_or_gone(port_name) {
                AttemptResult::Success
            } else {
                AttemptResult::Disconnected
            };
        }
        if let Some(resp) = read_response(port.as_mut(), 32) {
            if resp.windows(expected_ack.len()).any(|w| w == expected_ack) || resp.windows(5).any(|w| w == b"READY") {
                return AttemptResult::Success;
            }
        }
        if last_mode_check.elapsed() >= std::time::Duration::from_millis(500) {
            if !fastboot_devices().is_empty() {
                return AttemptResult::Success;
            }
            last_mode_check = std::time::Instant::now();
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    let mut last_write = std::time::Instant::now();
    let probe_start = std::time::Instant::now();
    while std::time::Instant::now().duration_since(probe_start) < probe_duration {
        if let Some(resp) = read_response(port.as_mut(), 64) {
            if resp.windows(expected_ack.len()).any(|w| w == expected_ack) || resp.windows(5).any(|w| w == b"READY") {
                return AttemptResult::Success;
            }
        }
        if last_write.elapsed() >= std::time::Duration::from_millis(200) {
            if port.write(BOOT_MODE_CMD).is_err() {
                return if in_fastboot_mode_or_gone(port_name) {
                    AttemptResult::Success
                } else {
                    AttemptResult::Disconnected
                };
            }
            last_write = std::time::Instant::now();
        }
        if last_mode_check.elapsed() >= std::time::Duration::from_millis(500) {
            if !fastboot_devices().is_empty() {
                return AttemptResult::Success;
            }
            last_mode_check = std::time::Instant::now();
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    AttemptResult::NoAck
}

fn read_response(port: &mut dyn serialport::SerialPort, size: usize) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; size];
    match port.read(buf.as_mut_slice()) {
        Ok(count) if count > 0 => {
            buf.truncate(count);
            Some(buf)
        }
        _ => None,
    }
}

/// After a failed write: success if the device is now in fastboot OR the VCOM
/// port vanished (the device switched modes and dropped the port).
fn in_fastboot_mode_or_gone(port_name: &str) -> bool {
    if !fastboot_devices().is_empty() {
        return true;
    }
    if let Ok(ports) = serialport::available_ports() {
        if ports.iter().all(|p| p.port_name != port_name) {
            return true;
        }
    }
    false
}

// ── Terminal ─────────────────────────────────────────────────

/// Emits one terminal output line to the renderer.
fn emit_term(app: &AppHandle, line: &str) {
    let _ = app.emit("term:output", serde_json::json!({ "line": line }));
}

/// Splits a command line into a program + args, honoring double quotes so a
/// drag-dropped file path with spaces survives intact. Also detects bare
/// Windows drive-letter paths (e.g. C:\Users\My PC\...) and joins their
/// segments so spaces in file paths do not cause incorrect splits.
fn split_command(input: &str) -> Vec<String> {
    fn is_drive_path_start(s: &str) -> bool {
        let bytes = s.as_bytes();
        bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
    }
    fn is_flag(s: &str) -> bool {
        s.starts_with('-')
    }

    // First pass: basic space-split respecting double quotes.
    let mut raw: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in input.trim().chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    raw.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        raw.push(current);
    }
    if raw.is_empty() {
        return raw;
    }

    // Second pass: merge tokens that form a Windows path with spaces.
    let mut merged: Vec<String> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if is_drive_path_start(&raw[i]) {
            let mut combined = raw[i].clone();
            i += 1;
            while i < raw.len() && !is_flag(&raw[i]) {
                combined.push(' ');
                combined.push_str(&raw[i]);
                i += 1;
            }
            merged.push(combined);
        } else {
            merged.push(raw[i].clone());
            i += 1;
        }
    }
    merged
}

/// Routes a bare tool name to its bundled/real path.
fn resolve_program(program: &str) -> String {
    let lower = program.to_lowercase();
    match lower.as_str() {
        "adb" => adb_path(),
        "fastboot" => fastboot_path(),
        "scrcpy" => scrcpy_path().join("scrcpy.exe").to_string_lossy().into_owned(),
        // Anything else runs from PATH (real cmd feel).
        _ => program.to_string(),
    }
}

/// Runs a command line typed in the toolkit terminal, streaming stdout+stderr
/// to the renderer line-by-line in real time. adb/fastboot resolve to the
/// bundled platform-tools; anything else falls through to PATH.
pub fn term_run(app: AppHandle, command: &str) -> Result<bool, String> {
    let parts = split_command(command);
    if parts.is_empty() {
        return Ok(true);
    }
    crate::log::write(&format!("terminal command: {}", command));
    let program = resolve_program(&parts[0]);
    let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

    let mut cmd = base_cmd(&program);
    cmd.args(&args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            emit_term(&app, &format!("error: could not run '{}': {e}", parts[0]));
            return Ok(false);
        }
    };

    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();

    if let Some(mut out) = stdout.take() {
        let app_stdout = app.clone();
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            let mut carry = String::new();
            loop {
                match out.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                        while let Some(pos) = carry.find('\n') {
                            let line = carry[..pos].trim_end_matches('\r').to_string();
                            if !line.is_empty() {
                                emit_term(&app_stdout, &line);
                            }
                            carry = carry[pos + 1..].to_string();
                        }
                    }
                    Err(_) => break,
                }
            }
            if !carry.trim().is_empty() {
                emit_term(&app_stdout, &carry.trim_end_matches('\r'));
            }
        });
    }
    drop(stdout);

    if let Some(mut err) = stderr.take() {
        let app_stderr = app.clone();
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            let mut carry = String::new();
            loop {
                match err.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                        while let Some(pos) = carry.find('\n') {
                            let line = carry[..pos].trim_end_matches('\r').to_string();
                            if !line.is_empty() {
                                emit_term(&app_stderr, &line);
                            }
                            carry = carry[pos + 1..].to_string();
                        }
                    }
                    Err(_) => break,
                }
            }
            if !carry.trim().is_empty() {
                emit_term(&app_stderr, &carry.trim_end_matches('\r'));
            }
        });
    }
    drop(stderr);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            emit_term(&app, "[timed out after 300s, killed]");
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let status = child.wait().unwrap_or_default();
    let code = status.code().unwrap_or(-1);
    if code != 0 {
        emit_term(&app, &format!("[exit code: {code}]"));
    }
    Ok(true)
}

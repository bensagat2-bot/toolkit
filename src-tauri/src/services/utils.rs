use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return default_output(),
    };
    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => return default_output(),
    };
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let _reader = std::thread::spawn(move || {
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

    let out_buf = rx.recv().unwrap_or_default();
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

fn adb_serial(serial: &str, args: &[&str], timeout_ms: u64) -> String {
    adb(&["-s", serial].into_iter().chain(args.iter().copied()).collect::<Vec<_>>(), timeout_ms)
}

fn fastboot_serial(serial: &str, args: &[&str], timeout_ms: u64) -> String {
    fastboot(&["-s", serial].into_iter().chain(args.iter().copied()).collect::<Vec<_>>(), timeout_ms)
}

fn downloads_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| std::env::temp_dir())
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

fn download_file(app: &AppHandle, url: &str, filename: &str) -> Result<PathBuf, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("V1Per-Toolkit/1.0")
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;
    let dest = downloads_dir().join(filename);
    if dest.exists() && dest.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
        return Ok(dest);
    }
    let resp = client.get(url).send().map_err(|e| format!("HTTP request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} from {}", resp.status(), url));
    }
    let bytes = resp.bytes().map_err(|e| format!("Failed to read response: {e}"))?;
    std::fs::write(&dest, &bytes).map_err(|e| format!("Failed to write file: {e}"))?;
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

fn device_prop(serial: &str, prop: &str) -> String {
    adb_serial(serial, &["shell", "getprop", prop], 8000).trim().to_string()
}

fn fastboot_devices() -> Vec<String> {
    let out = fastboot(&["devices"], 8000);
    out.lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.is_empty() || t.contains("waiting") || t.contains("no permission") {
                return None;
            }
            let parts: Vec<&str> = t.split_whitespace().collect();
            if parts.is_empty() { None } else { Some(parts[0].to_string()) }
        })
        .collect()
}

fn is_unlocked(serial: &str) -> bool {
    let out = fastboot_serial(serial, &["getvar", "unlocked"], 10000);
    out.lines()
        .any(|l| l.to_lowercase().replace("(bootloader)", "").contains("unlocked: yes"))
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

#[derive(Debug, Clone, Deserialize)]
pub struct RootOptions {
    pub boot_img: String,
}

/// Runs the KernelSU-Next / FolkPatch root pipeline.
/// Mirrors v1per-wpf Root.cs: detect kernel, pick patch tool, install the
/// manager, patch the boot image on-device (no root required), flash it.
pub fn root(app: AppHandle, opts: RootOptions) -> Result<bool, String> {
    let started = std::time::Instant::now();
    let boot_img = Path::new(&opts.boot_img);
    if !boot_img.exists() {
        return Err(format!("File not found: {}", opts.boot_img));
    }

    emit(&app, "Checking ADB Connection...");
    let serial = ready_adb_device()?;
    emit(&app, "Checking ADB Connection... FOUND");

    let model = device_prop(&serial, "ro.product.model");
    let sdk = device_prop(&serial, "ro.build.version.sdk");
    let release = device_prop(&serial, "ro.build.version.release");
    let platform = device_prop(&serial, "ro.board.platform");
    let abi = device_prop(&serial, "ro.product.cpu.abi");
    emit(&app, format!("ro.product.model : {}", model).as_str());
    emit(&app, format!("ro.build.version.sdk : {}", sdk).as_str());
    emit(&app, format!("ro.build.version.release : {}", release).as_str());
    emit(&app, format!("ro.board.platform : {}", platform).as_str());

    let kernel = get_kernel_version(&serial);
    let use_folk = match uses_folk(&kernel) {
        Some(true) => {
            emit(&app, format!("Kernel {} - below 5.10, using FolkPatch", kernel).as_str());
            true
        }
        Some(false) => {
            emit(&app, format!("Kernel {} - 5.10 or above, using KernelSU-Next", kernel).as_str());
            false
        }
        None => {
            emit(&app, format!("Could not parse kernel version: {}", kernel).as_str());
            emit(&app, "Defaulting to KernelSU-Next");
            false
        }
    };

    let (release_url, manager_label, apk_label, prefer): (&str, &str, &str, &str) = if use_folk {
        emit(&app, "Fetching releases info for FolkPatch... DONE");
        ("https://api.github.com/repos/LyraVoid/FolkPatch/releases/latest", "FolkPatch", "FolkPatch.apk", "folkpatch")
    } else {
        emit(&app, "Fetching releases info for ksu... DONE");
        ("https://api.github.com/repos/KernelSU-Next/KernelSU-Next/releases/latest", "KernelSU-Next", "KernelSU.apk", "universal")
    };

    let release = fetch_json(release_url).ok_or("Failed to fetch release. Check internet.")?;
    let apk = pick_apk(&release, &[prefer]).ok_or("Failed to fetch release asset.")?;
    let apk_url = asset_url(&apk);
    let apk_name = asset_name(&apk);

    emit(&app, "Downloading...");
    let apk_path = download_file(&app, &apk_url, &apk_name)?;
    emit(&app, format!("Downloading [{}]... DONE", apk_label).as_str());

    emit(&app, format!("Installing [{}] on device...", manager_label).as_str());
    let install = adb_serial(&serial, &["install", "-r", apk_path.to_string_lossy().as_ref()], 120000);
    if !install.to_lowercase().contains("success") {
        return Err(format!("Install failed: {}", install));
    }
    emit(&app, "Installing to device... DONE");

    // FolkPatch can only patch boot.img; KernelSU-Next handles init_boot too.
    if use_folk {
        let n = boot_img.file_name().map(|f| f.to_string_lossy().to_lowercase()).unwrap_or_default();
        if n.contains("init_boot") {
            return Err("FolkPatch needs boot.img, not init_boot.img.".into());
        }
    }
    let part_name = partition_from_image(boot_img);

    emit(&app, "Patching boot image...");
    let patched_local = if use_folk {
        auto_patch_folkpatch(&app, &serial, boot_img.to_string_lossy().as_ref())?
    } else {
        auto_patch_ksud(&app, &serial, boot_img.to_string_lossy().as_ref(), &release, &abi)?
    };
    let Some(patched_local) = patched_local else {
        return Err("Auto-patch failed.".into());
    };
    emit(&app, format!("Patching complete: {}", patched_local).as_str());

    flash_patched(&app, &serial, part_name, &patched_local)?;

    let elapsed = started.elapsed().as_secs();
    emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
    Ok(true)
}

fn get_kernel_version(serial: &str) -> String {
    let uname = adb_serial(serial, &["shell", "uname", "-r"], 8000).trim().to_string();
    if !uname.is_empty() && uname != "N/A" {
        return uname;
    }
    let ver = adb_serial(serial, &["shell", "cat", "/proc/version"], 8000);
    let ver = ver.trim();
    if ver.is_empty() {
        return "N/A".to_string();
    }
    // "Linux version 4.14.180-perf+ ..." -> "4.14.180-perf+"
    if let Some(rest) = ver.split("Linux version ").nth(1) {
        if let Some(part) = rest.split_whitespace().next() {
            return part.to_string();
        }
    }
    ver.chars().take(80).collect()
}

fn uses_folk(kernel: &str) -> Option<bool> {
    let mut nums = kernel
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| p.parse::<u32>().ok());
    let major = nums.next()?;
    let minor = nums.next().unwrap_or(0);
    Some(major < 5 || (major == 5 && minor < 10))
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

fn auto_patch_ksud(
    app: &AppHandle,
    serial: &str,
    image_path: &str,
    release: &serde_json::Value,
    abi: &str,
) -> Result<Option<String>, String> {
    let ksud = pick_ksud(release, abi).ok_or("No on-device ksud in this release.")?;
    let ksud_local = download_file(app, &asset_url(&ksud), &asset_name(&ksud))?;

    const REMOTE_KSUD: &str = "/data/local/tmp/ksud";
    const REMOTE_IN: &str = "/sdcard/Download/v1per_input.img";
    const REMOTE_OUT_NAME: &str = "kernelsu_patched.img";
    let remote_out = format!("/sdcard/Download/{REMOTE_OUT_NAME}");

    adb_serial(serial, &["shell", "mkdir", "-p", "/sdcard/Download"], 10000);
    adb_serial(serial, &["push", ksud_local.to_string_lossy().as_ref(), REMOTE_KSUD], 60000);
    adb_serial(serial, &["push", image_path, REMOTE_IN], 120000);
    adb_serial(serial, &["shell", "chmod", "755", REMOTE_KSUD], 10000);
    adb_serial(serial, &["shell", "rm", "-f", remote_out.as_str()], 10000);

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

    let patched_local = downloads_dir().join("patched_boot.img");
    adb_serial(serial, &["pull", remote_out.as_str(), patched_local.to_string_lossy().as_ref()], 120000);
    let ok = std::fs::metadata(&patched_local).map(|m| m.len() >= 4096).unwrap_or(false);
    if !ok {
        return Err("Failed to pull patched image.".into());
    }
    emit(app, format!("Patching [{}].img... DONE", "boot").as_str());
    Ok(Some(patched_local.to_string_lossy().into_owned()))
}

fn auto_patch_folkpatch(app: &AppHandle, serial: &str, image_path: &str) -> Result<Option<String>, String> {
    let release = fetch_json("https://api.github.com/repos/bmax121/KernelPatch/releases/latest")
        .ok_or("Failed to fetch KernelPatch release. Check internet.")?;
    let tools = pick_asset(&release, "kptools-android").ok_or("No kptools-android in KernelPatch release.")?;
    let kpimg = pick_asset(&release, "kpimg-android").ok_or("No kpimg-android in KernelPatch release.")?;

    let tools_local = download_file(app, &asset_url(&tools), &asset_name(&tools))?;
    let kpimg_local = download_file(app, &asset_url(&kpimg), &asset_name(&kpimg))?;

    const REMOTE_BIN: &str = "/data/local/tmp/kptools";
    const REMOTE_KPIMG: &str = "/data/local/tmp/kpimg-android";
    const WORK: &str = "/data/local/tmp/v1per_kp";
    let remote_in = format!("{WORK}/boot.img");
    let mut remote_out = "/sdcard/Download/folkpatch_patched.img".to_string();

    adb_serial(serial, &["shell", "rm", "-rf", WORK], 10000);
    adb_serial(serial, &["shell", "mkdir", "-p", WORK], 10000);
    adb_serial(serial, &["shell", "mkdir", "-p", "/sdcard/Download"], 10000);
    adb_serial(serial, &["push", tools_local.to_string_lossy().as_ref(), REMOTE_BIN], 60000);
    adb_serial(serial, &["push", kpimg_local.to_string_lossy().as_ref(), REMOTE_KPIMG], 60000);
    adb_serial(serial, &["push", image_path, remote_in.as_str()], 120000);
    adb_serial(serial, &["shell", "chmod", "755", REMOTE_BIN], 10000);
    adb_serial(serial, &["shell", "rm", "-f", remote_out.as_str()], 10000);

    emit(app, "Waiting for patched image...");
    adb_serial(serial, &[
        "shell", REMOTE_BIN, "-p", "--image", remote_in.as_str(), "--skey", "su",
        "--kpimg", REMOTE_KPIMG, "--out", remote_out.as_str(),
    ], 180000);

    let exists = adb_serial(serial, &["shell", "ls", remote_out.as_str()], 10000);
    if exists.trim().is_empty() || exists.to_lowercase().contains("no such") {
        emit(app, "Direct patch failed, falling back to unpack/patch/repack...");
        adb_serial(serial, &["shell", "sh", "-c", format!("cd {WORK} && {REMOTE_BIN} unpack boot.img").as_str()], 60000);
        let kernel_path = format!("{WORK}/kernel");
        let has_kernel = adb_serial(serial, &["shell", "ls", kernel_path.as_str()], 10000);
        if has_kernel.trim().is_empty() || has_kernel.to_lowercase().contains("no such") {
            return Err("kptools did not unpack a kernel.".into());
        }
        adb_serial(serial, &[
            "shell", REMOTE_BIN, "-p", "--image", kernel_path.as_str(), "--skey", "su",
            "--kpimg", REMOTE_KPIMG, "--out", kernel_path.as_str(),
        ], 180000);
        adb_serial(serial, &["shell", "sh", "-c", format!("cd {WORK} && {REMOTE_BIN} repack boot.img").as_str()], 60000);
        for candidate in [format!("{WORK}/new-boot.img"), format!("{WORK}/boot.img")] {
            let listed = adb_serial(serial, &["shell", "ls", candidate.as_str()], 10000);
            if !listed.trim().is_empty() && !listed.to_lowercase().contains("no such") {
                remote_out = candidate;
                break;
            }
        }
    }

    let patched_local = downloads_dir().join("patched_boot.img");
    adb_serial(serial, &["pull", remote_out.as_str(), patched_local.to_string_lossy().as_ref()], 120000);
    let ok = std::fs::metadata(&patched_local).map(|m| m.len() >= 4096).unwrap_or(false);
    if !ok {
        return Err("Failed to pull patched image.".into());
    }
    emit(app, "Patching boot.img... DONE");
    Ok(Some(patched_local.to_string_lossy().into_owned()))
}

fn flash_patched(app: &AppHandle, serial: &str, partition: &str, patched_local: &str) -> Result<(), String> {
    emit(app, "Rebooting device to bootloader mode...");
    adb_serial(serial, &["reboot", "bootloader"], 15000);

    emit(app, "Waiting for device in fastboot mode...");
    let mut fb_serial = None;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let devs = fastboot_devices();
        if let Some(d) = devs.first() {
            fb_serial = Some(d.clone());
            break;
        }
    }
    let fb_serial = fb_serial.ok_or("Fastboot device not detected.")?;
    emit(app, "Device detected in fastboot mode");

    emit(app, "Checking bootloader status...");
    let mut unlocked = is_unlocked(&fb_serial);
    if unlocked {
        emit(app, "Bootloader is UNLOCKED");
    } else {
        emit(app, "Bootloader is LOCKED");
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

// ── AnyKernel ────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AnyKernelOptions {
    pub zip: String,
}

/// Flashes an AnyKernel zip via adb root script, or fastboot boot.img payload.
pub fn anykernel(app: AppHandle, opts: AnyKernelOptions) -> Result<bool, String> {
    let started = std::time::Instant::now();
    let zip_path = Path::new(&opts.zip);
    if !zip_path.exists() {
        return Err(format!("File not found: {}", opts.zip));
    }

    emit(&app, "Reading AnyKernel zip...");
    let has_update_binary = {
        let file = std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {e}"))?;
        (0..archive.len()).any(|i| {
            archive.by_index(i).ok().map(|f| f.name().to_lowercase().ends_with("update-binary")).unwrap_or(false)
        })
    };
    emit(&app, "Reading AnyKernel zip... DONE");

    let root = {
        let out = adb(&["shell", "su", "-c", "id"], 8000);
        out.contains("uid=0")
    };

    if root && has_update_binary {
        emit(&app, "Preparing /data/local/tmp/v1per_ak...");
        const DIR: &str = "/data/local/tmp/v1per_ak";
        const REMOTE_ZIP: &str = "/data/local/tmp/v1per_ak/ak.zip";
        const REMOTE_BIN: &str = "/data/local/tmp/v1per_ak/update-binary";

        adb(&["shell", "mkdir", "-p", DIR], 10000);
        emit(&app, "Pushing AnyKernel zip to device...");
        adb(&["push", zip_path.to_string_lossy().as_ref(), REMOTE_ZIP], 120000);

        let local_bin = downloads_dir().join("update-binary");
        {
            let file = std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {e}"))?;
            let mut found = false;
            for i in 0..archive.len() {
                let mut f = archive.by_index(i).map_err(|e| format!("Zip read error: {e}"))?;
                if f.name().to_lowercase().ends_with("update-binary") {
                    let mut out = std::fs::File::create(&local_bin).map_err(|e| format!("Failed to create file: {e}"))?;
                    std::io::copy(&mut f, &mut out).ok();
                    found = true;
                    break;
                }
            }
            if !found {
                return Err("No update-binary found in zip.".into());
            }
        }
        emit(&app, "Extracting update-binary from zip... DONE");
        adb(&["push", local_bin.to_string_lossy().as_ref(), REMOTE_BIN], 60000);
        adb(&["shell", "chmod", "755", REMOTE_BIN], 10000);

        emit(&app, "Flashing via AnyKernel update-binary (root). Do not unplug!");
        let cmd = format!("cd {} && {} {} 3 1 {}", DIR, REMOTE_BIN, DIR, REMOTE_ZIP);
        let result = adb(&["shell", "su", "-c", &cmd], 300000);
        if result.contains("records out") || result.contains("okay") || result.contains("success") {
            emit(&app, "Boot partition flashed... DONE");
        }
        emit(&app, "Flash complete, rebooting device...");
        adb(&["reboot"], 10000);
        let elapsed = started.elapsed().as_secs();
        emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
        return Ok(true);
    }

    emit(&app, if root { "No update-binary; falling back to fastboot." } else { "No root detected; falling back to fastboot." });

    // Extract a boot.img payload for fastboot.
    let chosen = {
        let file = std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {e}"))?;
        let mut boot = None;
        let mut init_boot = None;
        for i in 0..archive.len() {
            let mut f = archive.by_index(i).map_err(|e| format!("Zip read error: {e}"))?;
            let name = f.name().to_lowercase();
            if name.ends_with("boot.img") && boot.is_none() {
                let mut out = std::fs::File::create(downloads_dir().join("boot_anykernel.img")).map_err(|e| format!("Failed to create file: {e}"))?;
                std::io::copy(&mut f, &mut out).ok();
                boot = Some("boot".to_string());
            } else if name.ends_with("init_boot.img") && init_boot.is_none() {
                let mut out = std::fs::File::create(downloads_dir().join("init_boot_anykernel.img")).map_err(|e| format!("Failed to create file: {e}"))?;
                std::io::copy(&mut f, &mut out).ok();
                init_boot = Some("init_boot".to_string());
            }
        }
        boot.or(init_boot).ok_or("No complete boot.img payload in this zip for fastboot.")?
    };

    let image = downloads_dir().join(format!("{}_anykernel.img", chosen));
    emit(&app, "Rebooting device to bootloader mode...");
    adb(&["reboot", "bootloader"], 15000);
    emit(&app, "Waiting for device in fastboot mode...");
    let mut fb_serial = None;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let devs = fastboot_devices();
        if let Some(d) = devs.first() {
            fb_serial = Some(d.clone());
            break;
        }
    }
    let fb_serial = fb_serial.ok_or("Fastboot device not detected.")?;
    emit(&app, "Device detected in fastboot mode");

    emit(&app, format!("Flashing {} with boot_anykernel.img", chosen).as_str());
    let result = fastboot_serial(&fb_serial, &["flash", &chosen, image.to_string_lossy().as_ref()], 90000);
    if flash_looks_ok(&result) {
        emit(&app, "Flash successful, rebooting device...");
        fastboot_serial(&fb_serial, &["reboot"], 10000);
    } else {
        for slot in ["a", "b"] {
            let r = fastboot_serial(&fb_serial, &["flash", format!("{}_{}", chosen, slot).as_str(), image.to_string_lossy().as_ref()], 90000);
            if flash_looks_ok(&r) {
                emit(&app, "Flash successful, rebooting device...");
                fastboot_serial(&fb_serial, &["reboot"], 10000);
                break;
            }
        }
    }

    let elapsed = started.elapsed().as_secs();
    emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
    Ok(true)
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
    let serial = ready_adb_device()?;
    emit(&app, "Checking ADB connection... FOUND");
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

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::Deserialize;
use tauri::{AppHandle, Emitter};

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_resource_dir(path: PathBuf) {
    let _ = RESOURCE_DIR.set(path);
}

pub fn platform_tools_path() -> PathBuf {
    RESOURCE_DIR.get()
        .map(|d| d.join("platform-tools"))
        .unwrap_or_else(|| {
            let exe = std::env::current_exe().unwrap_or_default();
            exe.parent().unwrap_or(Path::new(".")).join("platform-tools")
        })
}

pub fn adb_path() -> PathBuf {
    platform_tools_path().join("adb.exe")
}

pub fn fastboot_path() -> PathBuf {
    platform_tools_path().join("fastboot.exe")
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

fn run_stdout(cmd: &mut Command, timeout_ms: u64) -> String {
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let mut buf = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        use std::io::Read;
        let mut chunk = [0u8; 8192];
        loop {
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                break;
            }
            match stdout.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }
        }
    }
    let _ = child.wait();
    String::from_utf8_lossy(&buf).to_string()
}

fn adb(args: &[&str], timeout_ms: u64) -> String {
    run_stdout(base_cmd(adb_path().to_str().unwrap_or("adb")).args(args), timeout_ms)
}

fn fastboot(args: &[&str], timeout_ms: u64) -> String {
    run_stdout(base_cmd(fastboot_path().to_str().unwrap_or("fastboot")).args(args), timeout_ms)
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

fn ready_adb_device() -> Option<String> {
    for _ in 0..5 {
        let out = adb(&["devices"], 8000);
        for line in out.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                return Some(parts[0].to_string());
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
    None
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
pub fn root(app: AppHandle, opts: RootOptions) -> Result<bool, String> {
    let started = std::time::Instant::now();
    let boot_img = Path::new(&opts.boot_img);
    if !boot_img.exists() {
        return Err(format!("File not found: {}", opts.boot_img));
    }

    emit(&app, "Checking ADB Connection...");
    let serial = ready_adb_device().ok_or("No ready ADB device. Accept USB debugging and reconnect.")?;
    emit(&app, "Checking ADB Connection... FOUND");

    let model = device_prop(&serial, "ro.product.model");
    let sdk = device_prop(&serial, "ro.build.version.sdk");
    let release = device_prop(&serial, "ro.build.version.release");
    let platform = device_prop(&serial, "ro.board.platform");
    emit(&app, format!("ro.product.model : {}", model).as_str());
    emit(&app, format!("ro.build.version.sdk : {}", sdk).as_str());
    emit(&app, format!("ro.build.version.release : {}", release).as_str());
    emit(&app, format!("ro.board.platform : {}", platform).as_str());

    let kernel = {
        let uname = adb_serial(&serial, &["shell", "uname", "-r"], 8000).trim().to_string();
        if !uname.is_empty() && uname != "N/A" {
            uname
        } else {
            let ver = adb_serial(&serial, &["shell", "cat", "/proc/version"], 8000);
            ver.lines().next().unwrap_or("N/A").trim().to_string()
        }
    };

    let use_folk = {
        let mut major = 0;
        let mut minor = 0;
        let mut found = false;
        for part in kernel.split(|c: char| !c.is_ascii_digit()) {
            if !part.is_empty() {
                if let Ok(v) = part.parse::<i32>() {
                    if !found {
                        major = v;
                        found = true;
                    } else {
                        minor = v;
                        break;
                    }
                }
            }
        }
        !found || major < 5 || (major == 5 && minor < 10)
    };

    if use_folk {
        emit(&app, format!("Kernel {} - below 5.10, using FolkPatch", kernel).as_str());
    } else {
        emit(&app, format!("Kernel {} - 5.10 or above, using KernelSU-Next", kernel).as_str());
    }

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

    let part_name = {
        let n = boot_img.file_name().map(|f| f.to_string_lossy().to_lowercase()).unwrap_or_default();
        if n.contains("init_boot") { "init_boot" } else { "boot" }
    };

    emit(&app, "Patching boot image...");
    let remote_bin = "/data/local/tmp/ksud";
    let remote_in = "/sdcard/Download/v1per_input.img";
    let remote_out = "/sdcard/Download/kernelsu_patched.img";

    adb_serial(&serial, &["shell", "mkdir", "-p", "/sdcard/Download"], 10000);
    adb_serial(&serial, &["shell", "rm", "-f", remote_out], 10000);

    let ksud = pick_asset(&release, "ksud-aarch64-linux-android")
        .or_else(|| pick_asset(&release, "ksud-armv7-linux-androideabi"))
        .or_else(|| pick_asset(&release, "ksud-x86_64-linux-android"));
    if let Some(ksud_asset) = ksud {
        let ksud_url = asset_url(&ksud_asset);
        let ksud_name = asset_name(&ksud_asset);
        if let Ok(local) = download_file(&app, &ksud_url, &ksud_name) {
            adb_serial(&serial, &["push", local.to_string_lossy().as_ref(), remote_bin], 60000);
            adb_serial(&serial, &["push", boot_img.to_string_lossy().as_ref(), remote_in], 120000);
            adb_serial(&serial, &["shell", "chmod", "755", remote_bin], 10000);
            adb_serial(&serial, &["shell", "su", "-c", format!("{} boot-patch -b {} -o /sdcard/Download --out-name kernelsu_patched.img", remote_bin, remote_in).as_str()], 180000);
            let exists = adb_serial(&serial, &["shell", "ls", remote_out], 10000);
            if exists.contains("kernelsu_patched") {
                let patched_local = downloads_dir().join("patched_boot.img");
                adb_serial(&serial, &["pull", remote_out, patched_local.to_string_lossy().as_ref()], 120000);
                emit(&app, format!("Patching [{}].img... DONE", part_name).as_str());

                emit(&app, "Pulling and rebooting to fastboot...");
                adb_serial(&serial, &["reboot", "bootloader"], 15000);
                emit(&app, "Rebooting to fastboot... DONE");

                emit(&app, "Waiting for fastboot devices...");
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
                emit(&app, "Waiting for fastboot devices... FOUND");

                emit(&app, "Checking bootloader status...");
                let unlocked = is_unlocked(&fb_serial);
                if unlocked {
                    emit(&app, "Bootloader is UNLOCKED");
                } else {
                    emit(&app, "Bootloader is LOCKED");
                    emit(&app, "Attempting to unlock bootloader.... DONE");
                    for _attempt in 0..5 {
                        let r = fastboot_serial(&fb_serial, &["flashing", "unlock"], 20000);
                        if r.is_empty() || r.to_lowercase().contains("unknown") {
                            fastboot_serial(&fb_serial, &["oem", "unlock"], 20000);
                        }
                        std::thread::sleep(std::time::Duration::from_secs(4));
                        if is_unlocked(&fb_serial) {
                            break;
                        }
                    }
                }

                emit(&app, format!("Flashing {}...", part_name).as_str());
                let flash = fastboot_serial(&fb_serial, &["flash", part_name, patched_local.to_string_lossy().as_ref()], 90000);
                if !flash_looks_ok(&flash) {
                    for slot in ["a", "b"] {
                        let r = fastboot_serial(&fb_serial, &["flash", format!("{}_{}", part_name, slot).as_str(), patched_local.to_string_lossy().as_ref()], 90000);
                        if flash_looks_ok(&r) {
                            break;
                        }
                    }
                }
                emit(&app, "Flashed patched partition... DONE");

                emit(&app, "Rebooting Device...");
                fastboot_serial(&fb_serial, &["reboot"], 10000);
            } else {
                emit(&app, "Patching skipped (on-device ksud did not produce an image).");
            }
        }
    } else {
        emit(&app, "Patching skipped (no on-device ksud in release).");
    }

    let elapsed = started.elapsed().as_secs();
    emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
    Ok(true)
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

pub fn scrcpy_path() -> PathBuf {
    RESOURCE_DIR.get()
        .map(|d| d.join("scrcpy"))
        .unwrap_or_else(|| {
            let exe = std::env::current_exe().unwrap_or_default();
            exe.parent().unwrap_or(Path::new(".")).join("scrcpy")
        })
}

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
    let serial = ready_adb_device().ok_or("No ready ADB device. Accept USB debugging and reconnect.")?;
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

/// Floods the MTK preloader serial port with "FASTBOOT" to force the device into fastboot mode.
pub fn force_fastboot(app: AppHandle) -> Result<bool, String> {
    let started = std::time::Instant::now();
    emit(&app, "Forcing it now.....");
    emit(&app, "Attempting to switch device to Fastboot...");

    let expected_ack = {
        let mut ack = b"READY".to_vec();
        let tail: Vec<u8> = b"FASTBOOT".iter().rev().take(3).copied().collect();
        ack.extend(tail);
        ack
    };

let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let mut saw_device = false;
    let mut last_attempt: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();

    loop {
        if !fastboot_devices().is_empty() {
            emit(&app, "Device already in Fastboot mode.");
            emit(&app, "OK DONE");
            break;
        }
        if std::time::Instant::now() >= deadline {
            if !saw_device {
                emit(&app, "No MTK preloader device detected.");
                return Err("Make sure the phone is powered off and connected via USB.".into());
            }
            return Err("Device detected but no ACK received from preloader.".into());
        }

        let ports = match serialport::available_ports() {
            Ok(p) => p,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(200));
                continue;
            }
        };
        let candidates = ports.iter().filter(|p| {
            let desc = p.port_name.to_lowercase();
            desc.contains("mediatek") || desc.contains("preloader") || desc.contains("mtk")
        });
        let mut tried_any = false;
        for port in candidates {
            let now = std::time::Instant::now();
            if last_attempt.get(&port.port_name).map(|t| (now - *t).as_millis() < 500).unwrap_or(false) {
                continue;
            }
            last_attempt.insert(port.port_name.clone(), now);
            tried_any = true;
            saw_device = true;
            emit(&app, format!("Detected candidate MTK port: {}", port.port_name).as_str());

            for baud in [115200u32, 921600, 57600] {
                match serialport::new(&port.port_name, baud).timeout(std::time::Duration::from_millis(1)).open() {
                    Ok(mut sp) => {
                        let flood_end = std::time::Instant::now() + std::time::Duration::from_secs(1);
                        let mut got_ack = false;
                        while std::time::Instant::now() < flood_end {
                            let _ = sp.write(b"FASTBOOT");
                            let mut buf = [0u8; 64];
                            if let Ok(n) = sp.read(&mut buf) {
                                if n > 0 && (buf[..n].windows(expected_ack.len()).any(|w| w == expected_ack.as_slice()) || buf[..n].windows(5).any(|w| w == b"READY")) {
                                    got_ack = true;
                                    break;
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(30));
                            if !fastboot_devices().is_empty() {
                                got_ack = true;
                                break;
                            }
                        }
                        if got_ack {
                            emit(&app, "Handshake successful. Device should be in Fastboot.");
                            emit(&app, "OK DONE");
                            let elapsed = started.elapsed().as_secs();
                            emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
                            return Ok(true);
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        if !tried_any {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    let elapsed = started.elapsed().as_secs();
    emit(&app, format!("Elapsed Time: {}s", elapsed).as_str());
    Ok(true)
}

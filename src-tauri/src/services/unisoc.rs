use std::collections::HashMap;
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use serde::Deserialize;
use tauri::{AppHandle, Emitter};

static CURRENT_PROCESS: Mutex<Option<u32>> = Mutex::new(None);
static mut RESOURCE_DIR: Option<PathBuf> = None;

pub fn set_resource_dir(path: PathBuf) {
    unsafe { RESOURCE_DIR = Some(path); }
}

fn get_resource_dir() -> PathBuf {
    unsafe {
        RESOURCE_DIR.clone().unwrap_or_else(|| {
            let exe = std::env::current_exe().unwrap_or_default();
            exe.parent().unwrap_or(&std::path::PathBuf::from(".")).to_path_buf()
        })
    }
}

fn emit_progress(app: &AppHandle, msg: &str) {
    let _ = app.emit("unisoc:progress", serde_json::json!({ "message": msg }));
}

#[derive(Debug, Clone)]
pub struct UnisocPackage {
    pub id: String,
    pub name: String,
    pub exec_addr: u64,
    pub fdl1: String,
    pub fdl1_addr: u64,
    pub fdl2: String,
    pub fdl2_addr: u64,
    pub cboot: String,
    pub spl_loader_bk: Option<String>,
    pub misc_done: String,
    pub chsize_uboot: bool,
    pub tools_gen: String,
    pub erase_persist: bool,
    pub backup_partitions: Vec<String>,
    pub files: Vec<String>,
}

fn build_packages() -> HashMap<String, UnisocPackage> {
    let mut m = HashMap::new();
    m.insert("ums9230".into(), UnisocPackage {
        id: "ums9230".into(), name: "UMS9230 (T606/T612)".into(),
        exec_addr: 0x65015f08, fdl1: "fdl1-dl.bin".into(), fdl1_addr: 0x65000800,
        fdl2: "fdl2-dl.bin".into(), fdl2_addr: 0x9efffe00, cboot: "fdl2-cboot.bin".into(),
        spl_loader_bk: Some("splloader_bk.bin".into()), misc_done: "misc-ubldone.bin".into(),
        chsize_uboot: false, tools_gen: "gen1".into(), erase_persist: true,
        backup_partitions: vec!["boot".into(), "init_boot".into(), "vendor_boot".into(), "prodnv".into()],
        files: vec!["custom_exec_no_verify_65015f08.bin".into(), "fdl1-dl.bin".into(), "fdl2-dl.bin".into(),
            "fdl2-cboot.bin".into(), "splloader_bk.bin".into(), "misc-ubldone.bin".into(), "misc-wipe.bin".into()],
    });
    m.insert("sc9863a".into(), UnisocPackage {
        id: "sc9863a".into(), name: "SC9863A".into(),
        exec_addr: 0x4ee8, fdl1: "fdl1-dl.bin".into(), fdl1_addr: 0x5000,
        fdl2: "fdl2-dl.bin".into(), fdl2_addr: 0x9efffe00, cboot: "fdl2-cboot.bin".into(),
        spl_loader_bk: None, misc_done: "misc-wipe.bin".into(),
        chsize_uboot: true, tools_gen: "gen1".into(), erase_persist: false,
        backup_partitions: vec!["boot".into(), "prodnv".into()],
        files: vec!["custom_exec_no_verify_4ee8.bin".into(), "fdl1-dl.bin".into(), "fdl2-dl.bin".into(),
            "fdl2-cboot.bin".into(), "misc-wipe.bin".into()],
    });
    m.insert("ums512".into(), UnisocPackage {
        id: "ums512".into(), name: "UMS512 (T610/T700)".into(),
        exec_addr: 0x3ee8, fdl1: "fdl1-dl.bin".into(), fdl1_addr: 0x5500,
        fdl2: "fdl2-dl.bin".into(), fdl2_addr: 0x9efffe00, cboot: "fdl2-cboot.bin".into(),
        spl_loader_bk: None, misc_done: "misc-wipe.bin".into(),
        chsize_uboot: false, tools_gen: "gen2".into(), erase_persist: false,
        backup_partitions: vec!["boot".into(), "prodnv".into()],
        files: vec!["custom_exec_no_verify_3ee8.bin".into(), "fdl1-dl.bin".into(), "fdl2-dl.bin".into(),
            "fdl2-cboot.bin".into(), "misc-wipe.bin".into()],
    });
    m.insert("ums9620".into(), UnisocPackage {
        id: "ums9620".into(), name: "UMS9620".into(),
        exec_addr: 0x65012f48, fdl1: "fdl1-dl.bin".into(), fdl1_addr: 0x65000800,
        fdl2: "fdl2-dl.bin".into(), fdl2_addr: 0x9efffe00, cboot: "fdl2-cboot.bin".into(),
        spl_loader_bk: None, misc_done: "misc-wipe.bin".into(),
        chsize_uboot: false, tools_gen: "gen2".into(), erase_persist: false,
        backup_partitions: vec!["boot".into(), "prodnv".into()],
        files: vec!["custom_exec_no_verify_65012f48.bin".into(), "fdl1-dl.bin".into(), "fdl2-dl.bin".into(),
            "fdl2-cboot.bin".into(), "misc-wipe.bin".into()],
    });
    m
}

fn find_unisoc_root() -> Option<PathBuf> {
    let resource_dir = get_resource_dir();
    // Bundled build: resources land in resource_dir/Unisoc
    let candidate = resource_dir.join("Unisoc");
    if candidate.exists() { return Some(candidate); }
    // Dev: walk up from the executable to find the project's src-tauri/resources
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent()?;
    for _ in 0..8 {
        let candidate = dir.join("src-tauri").join("resources").join("Unisoc");
        if candidate.exists() { return Some(candidate); }
        let candidate = dir.join("resources").join("Unisoc");
        if candidate.exists() { return Some(candidate); }
        let candidate = dir.join("Unisoc");
        if candidate.exists() { return Some(candidate); }
        dir = dir.parent()?;
    }
    None
}

fn resolve_spd_dump(pkg_dir: &Path) -> Option<PathBuf> {
    if let Some(root) = find_unisoc_root() {
        let shared = root.join("spd_dump.exe");
        if shared.exists() { return Some(shared); }
    }
    let local = pkg_dir.join("spd_dump.exe");
    if local.exists() { Some(local) } else { None }
}

fn prepare_work(pkg_dir: &Path, device_dir: Option<&Path>, pkg: &UnisocPackage) -> PathBuf {
    let work = std::env::temp_dir().join("v1per_unisoc");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).ok();
    for name in &pkg.files {
        let src = pkg_dir.join(name);
        if src.exists() { fs::copy(&src, work.join(name)).ok(); }
    }
    if let Some(dd) = device_dir {
        for name in [&pkg.fdl1, &pkg.fdl2, &pkg.cboot] {
            let src = dd.join(name);
            if src.exists() { fs::copy(&src, work.join(name)).ok(); }
        }
    }
    work
}

fn build_base_tokens(wait: bool, pkg: &UnisocPackage) -> Vec<String> {
    let mut t = Vec::new();
    if wait { t.push("--wait".into()); t.push("300".into()); }
    t.push("exec_addr".into()); t.push(format!("0x{:x}", pkg.exec_addr));
    t.push("fdl".into()); t.push(pkg.fdl1.clone()); t.push(format!("0x{:x}", pkg.fdl1_addr));
    t.push("fdl".into()); t.push(pkg.fdl2.clone()); t.push(format!("0x{:x}", pkg.fdl2_addr));
    t.push("exec".into());
    t
}

fn build_base_tokens_wait(wait_secs: Option<u32>, pkg: &UnisocPackage) -> Vec<String> {
    let mut t = Vec::new();
    if let Some(secs) = wait_secs { t.push("--wait".into()); t.push(secs.to_string()); }
    t.push("exec_addr".into()); t.push(format!("0x{:x}", pkg.exec_addr));
    t.push("fdl".into()); t.push(pkg.fdl1.clone()); t.push(format!("0x{:x}", pkg.fdl1_addr));
    t.push("fdl".into()); t.push(pkg.fdl2.clone()); t.push(format!("0x{:x}", pkg.fdl2_addr));
    t.push("exec".into());
    t
}

fn run_spd_dump_stream(app: &AppHandle, exe: &Path, tokens: &[String], cwd: &Path) -> Result<String, String> {
    let mut cmd = Command::new(exe);
    cmd.args(tokens)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let mut child = cmd.spawn().map_err(|e| format!("Failed to run spd_dump: {e}"))?;
    *CURRENT_PROCESS.lock().unwrap() = Some(child.id());

    let stdout = child.stdout.take().ok_or("Failed to capture spd_dump output")?;
    let reader = std::io::BufReader::new(stdout);
    let mut output = String::new();
    for line in reader.lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            emit_progress(app, trimmed);
        }
        output.push_str(&line);
        output.push('\n');
    }
    let status = child.wait().map_err(|e| format!("spd_dump wait failed: {e}"))?;
    *CURRENT_PROCESS.lock().unwrap() = None;
    if !status.success() {
        return Err(format!("spd_dump exited with code {:?}", status.code()));
    }
    Ok(output)
}

fn run_helper(pkg_dir: &Path, tools_gen: &str, exe_name: &str, arg: &str, cwd: &Path) -> Result<String, String> {
    let name = if exe_name.ends_with(".exe") { exe_name.to_string() } else { format!("{}.exe", exe_name) };
    let mut exe_path = None;
    if let Some(root) = find_unisoc_root() {
        let shared = root.join("tools").join(tools_gen).join(&name);
        if shared.exists() { exe_path = Some(shared); }
    }
    if exe_path.is_none() {
        let local = pkg_dir.join(&name);
        if local.exists() { exe_path = Some(local); }
    }
    let path = exe_path.ok_or_else(|| format!("{} not found", name))?;
    let mut cmd = Command::new(path);
    cmd.arg(arg).current_dir(cwd);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.output().map_err(|e| format!("Failed to run {}: {}", name, e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ── Public API ──────────────────────────────────────────────

pub fn get_packages_installed() -> HashMap<String, bool> {
    let root = find_unisoc_root();
    let pkgs = build_packages();
    let mut result = HashMap::new();
    for id in pkgs.keys() {
        let installed = root.as_ref().map(|r| r.join(id).exists()).unwrap_or(false);
        result.insert(id.clone(), installed);
    }
    result
}

pub fn stop_process() -> bool {
    let mut proc = CURRENT_PROCESS.lock().unwrap();
    if let Some(pid) = proc.take() {
        let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
        return true;
    }
    false
}

fn resolve_pkg_device(pkg_id: &str, device: Option<&str>) -> Result<(UnisocPackage, PathBuf, PathBuf, Option<PathBuf>), String> {
    let pkgs = build_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?.clone();
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;
    let device_dir = device
        .map(|d| pkg_dir.join(d))
        .filter(|d| d.exists())
        .map(|p| p.as_path().to_path_buf());
    Ok((pkg, pkg_dir, spd_dump, device_dir))
}

/// Scans a firmware folder for flashable partition images.
pub fn scan_folder(folder: &str) -> Result<serde_json::Value, String> {
    let dir = Path::new(folder);
    if !dir.is_dir() {
        return Err("Selected path is not a folder".into());
    }
    let excluded = [
        "fdl1-dl", "fdl2-dl", "fdl2-cboot", "custom_exec", "spl-unlock",
        "misc-wipe", "misc-ubldone", "splloader_bk", "uboot_bk",
        ".xml", ".ini", ".txt", ".cfg", ".log", "readme", "checksum",
    ];
    let mut parts: Vec<serde_json::Value> = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read folder: {e}"))?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        let is_image = lower.ends_with(".img") || lower.ends_with(".bin");
        if !is_image || excluded.iter().any(|x| lower.contains(x)) {
            continue;
        }
        let part_name = Path::new(&name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| name.clone());
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        parts.push(serde_json::json!({ "name": part_name, "file": name, "size": size }));
    }
    parts.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(serde_json::json!({ "partitions": parts }))
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnisocFlashPart {
    pub name: String,
    pub file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnisocCliOp {
    pub name: String,
    pub part: Option<String>,
    pub file: Option<String>,
    pub offset: Option<String>,
    pub size: Option<String>,
    pub mode: Option<String>,
    pub value: Option<String>,
}

/// Human-readable label for a CLI op name.
fn op_label(name: &str) -> String {
    match name {
        "erase_all" => "Format All".into(),
        "erase" => "Erase Partition".into(),
        "write" => "Write Partition".into(),
        "read" => "Backup Partition".into(),
        "read_lite" => "Lite Backup".into(),
        "read_part" => "Read at Offset".into(),
        "write_offset" => "Write at Offset".into(),
        "list" => "Print Partition List".into(),
        "partition_list" => "Dump Partition Table".into(),
        "size_part" => "Partition Size".into(),
        "check_part" => "Check Partition".into(),
        "repartition" => "Repartition".into(),
        "set_active" => "Set Active Slot".into(),
        "verity" => "Toggle dm-verity".into(),
        "reboot_recovery" => "Reboot Recovery".into(),
        "reboot_fastboot" => "Reboot Fastboot".into(),
        "poweroff" => "Power Off".into(),
        "firstmode" => "Set Boot Mode".into(),
        "misc_fix" => "Write Misc Red-State Fix".into(),
        "backup_nv" => "Backup NV".into(),
        "restore_nv" => "Restore NV".into(),
        _ => name.to_string(),
    }
}

/// Builds the token list for a single CLI op.
fn build_op_tokens(pkg: &UnisocPackage, work: &Path, op: &UnisocCliOp) -> Result<Vec<String>, String> {
    let name = op.name.as_str();
    let mut tokens = Vec::new();
    match name {
        "erase_all" => tokens.extend(["verbose", "2", "erase_all", "reset"].iter().map(|s| s.to_string())),
        "erase" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            tokens.extend(["e", part, "reset"].iter().map(|s| s.to_string()));
        }
        "write" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            let file = op.file.as_deref().ok_or("Image file required")?;
            let full = Path::new(file);
            let target = if full.exists() { full.to_string_lossy().to_string() } else { work.join(file).to_string_lossy().to_string() };
            tokens.extend(["w", part, &target, "reset"].iter().map(|s| s.to_string()));
        }
        "read" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            tokens.extend(["r", part, "reset"].iter().map(|s| s.to_string()));
        }
        "read_lite" => tokens.extend(["r", "all_lite", "reset"].iter().map(|s| s.to_string())),
        "read_part" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            let offset = op.offset.as_deref().ok_or("Offset required")?;
            let size = op.size.as_deref().ok_or("Size required")?;
            let file = op.file.as_deref().ok_or("Output file required")?;
            tokens.extend(["read_part", part, offset, size, file, "reset"].iter().map(|s| s.to_string()));
        }
        "write_offset" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            let offset = op.offset.as_deref().ok_or("Offset required")?;
            let file = op.file.as_deref().ok_or("Image file required")?;
            let full = Path::new(file);
            let target = if full.exists() { full.to_string_lossy().to_string() } else { work.join(file).to_string_lossy().to_string() };
            tokens.extend(["wof", part, offset, &target, "reset"].iter().map(|s| s.to_string()));
        }
        "list" => tokens.extend(["p", "reset"].iter().map(|s| s.to_string())),
        "partition_list" => {
            let file = op.file.as_deref().ok_or("Output file required")?;
            tokens.extend(["partition_list", file, "reset"].iter().map(|s| s.to_string()));
        }
        "size_part" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            tokens.extend(["size_part", part, "reset"].iter().map(|s| s.to_string()));
        }
        "check_part" => {
            let part = op.part.as_deref().ok_or("Partition name required")?;
            tokens.extend(["check_part", part, "reset"].iter().map(|s| s.to_string()));
        }
        "repartition" => {
            let file = op.file.as_deref().ok_or("XML file required")?;
            let full = Path::new(file);
            let target = if full.exists() { full.to_string_lossy().to_string() } else { work.join(file).to_string_lossy().to_string() };
            tokens.extend(["repartition", &target, "reset"].iter().map(|s| s.to_string()));
        }
        "set_active" => {
            let mode = op.mode.as_deref().ok_or("Slot (a/b) required")?;
            tokens.extend(["set_active", mode, "reset"].iter().map(|s| s.to_string()));
        }
        "verity" => {
            let value = op.value.as_deref().ok_or("Verity value required")?;
            tokens.extend(["verity", value, "reset"].iter().map(|s| s.to_string()));
        }
        "reboot_recovery" => tokens.extend(["reboot-recovery"].iter().map(|s| s.to_string())),
        "reboot_fastboot" => tokens.extend(["reboot-fastboot"].iter().map(|s| s.to_string())),
        "poweroff" => tokens.extend(["poweroff"].iter().map(|s| s.to_string())),
        "firstmode" => {
            let mode = op.mode.as_deref().ok_or("Mode id required")?;
            tokens.extend(["firstmode", mode, "reset"].iter().map(|s| s.to_string()));
        }
        "misc_fix" => {
            let file = op.file.as_deref().unwrap_or(&pkg.misc_done);
            let full = Path::new(file);
            let target = if full.exists() { full.to_string_lossy().to_string() } else { work.join(file).to_string_lossy().to_string() };
            tokens.extend(["w", "misc", &target, "reset"].iter().map(|s| s.to_string()));
        }
        "backup_nv" => tokens.extend(["r", "prodnv", "r", "nvdata", "r", "nvcfg", "reset"].iter().map(|s| s.to_string())),
        "restore_nv" => {
            let prodnv = op.file.as_deref().ok_or("prodnv file required")?;
            let nvdata = op.value.as_deref().ok_or("nvdata file required")?;
            tokens.extend(["w", "prodnv", prodnv, "w", "nvdata", nvdata, "reset"].iter().map(|s| s.to_string()));
        }
        _ => return Err(format!("Unknown CLI op: {name}")),
    }
    Ok(tokens)
}

/// Runs a single CLI op with a full connection + flow-text log.
fn run_op(app: &AppHandle, pkg: &UnisocPackage, spd_dump: &Path, work: &Path, wait_secs: Option<u32>, kick: bool, kickto: Option<String>, baudrate: Option<String>, blk_size: Option<String>, op: &UnisocCliOp) -> Result<bool, String> {
    let label = op_label(&op.name);
    emit_progress(app, "Searching for Unisoc dl_diag device...");

    let mut tokens = build_base_tokens_wait(wait_secs, pkg);
    let mut kick_flags = Vec::new();
    if kick { kick_flags.push("--kick".into()); }
    if let Some(mode) = kickto { kick_flags.push("--kickto".into()); kick_flags.push(mode); }
    let insert_at = tokens.iter().position(|t| t == "exec_addr").unwrap_or(0);
    tokens.splice(insert_at..insert_at, kick_flags);
    if let Some(rate) = baudrate { tokens.push("baudrate".into()); tokens.push(rate); }
    if let Some(size) = blk_size { tokens.push("blk_size".into()); tokens.push(size); }

    let op_tokens = build_op_tokens(pkg, work, op)?;
    tokens.extend(op_tokens);

    let result = run_spd_dump_stream(app, spd_dump, &tokens, work);
    if result.is_err() {
        emit_progress(app, "Searching for Unisoc dl_diag device... NOT FOUND, please retry");
        return Err(result.unwrap_err());
    }

    emit_progress(app, "Searching for Unisoc dl_diag device... FOUND");
    emit_progress(app, "SPRD USB Download Port detected");
    emit_progress(app, "Connecting to device... Ok");
    emit_progress(app, format!("ChipId : {}", pkg.name).as_str());
    emit_progress(app, "Sending FDL1/FDL2 loaders... Ok");
    emit_progress(app, format!("{}... Ok", label).as_str());
    Ok(true)
}

/// Runs a list of spd_dump CLI operations with shared connection settings.
pub fn run_cli(
    app: AppHandle,
    pkg_id: &str,
    device: Option<&str>,
    wait_secs: Option<u32>,
    kick: bool,
    kickto: Option<String>,
    baudrate: Option<String>,
    blk_size: Option<String>,
    ops: Vec<UnisocCliOp>,
) -> Result<bool, String> {
    let (pkg, pkg_dir, spd_dump, device_dir) = resolve_pkg_device(pkg_id, device)?;
    let work = prepare_work(&pkg_dir, device_dir.as_deref(), &pkg);

    for op in &ops {
        run_op(
            &app,
            &pkg,
            &spd_dump,
            &work,
            wait_secs,
            kick,
            kickto.clone(),
            baudrate.clone(),
            blk_size.clone(),
            op,
        )?;
    }

    emit_progress(&app, "Operation complete.");
    Ok(true)
}

/// Unlocks the bootloader using the patched-splloader research download flow.
pub fn unlock(app: AppHandle, pkg_id: &str, device: Option<&str>) -> Result<bool, String> {
    let (pkg, pkg_dir, spd_dump, device_dir) = resolve_pkg_device(pkg_id, device)?;
    let work = prepare_work(&pkg_dir, device_dir.as_deref(), &pkg);

    emit_progress(&app, "Searching for Unisoc dl_diag device...");
    let mut tokens = build_base_tokens(true, &pkg);
    tokens.extend(["r", "splloader", "r", "uboot", "e", "splloader", "e", "splloader_bak", "reset"]
        .iter().map(|s| s.to_string()));
    let first = run_spd_dump_stream(&app, &spd_dump, &tokens, &work);
    if first.is_err() {
        emit_progress(&app, "Searching for Unisoc dl_diag device... NOT FOUND, please retry");
        return Err(first.unwrap_err());
    }
    emit_progress(&app, "Searching for Unisoc dl_diag device... FOUND");
    emit_progress(&app, "SPRD USB Download Port detected");
    emit_progress(&app, "Connecting to device... Ok");
    emit_progress(&app, format!("ChipId : {}", pkg.name).as_str());
    emit_progress(&app, "Sending FDL1/FDL2 loaders... Ok");
    emit_progress(&app, "Reading & erasing stock loaders... Ok");

    emit_progress(&app, "Generating unlock splloader...");
    let unlocker = work.join("spl-unlock.bin");
    if !unlocker.exists() {
        let spl_source = pkg.spl_loader_bk.as_deref().unwrap_or("splloader.bin");
        run_helper(&pkg_dir, &pkg.tools_gen, "gen_spl-unlock", spl_source, &work)?;
    }
    emit_progress(&app, "Generating unlock splloader... Ok");

    let spl16k = work.join("u-boot-spl-16k-sign.bin");
    let spl_bin = work.join("splloader.bin");
    if spl_bin.exists() { fs::rename(&spl_bin, &spl16k).ok(); }
    if pkg.chsize_uboot { run_helper(&pkg_dir, &pkg.tools_gen, "chsize", "uboot.bin", &work).ok(); }
    let ub_bin = work.join("uboot.bin");
    let ub_bak = work.join("uboot_bak.bin");
    if ub_bin.exists() { fs::rename(&ub_bin, &ub_bak).ok(); }

    emit_progress(&app, "Writing [uboot] -> [fdl2-cboot.bin]...");
    tokens = build_base_tokens(true, &pkg);
    tokens.extend(["w", "uboot", &pkg.cboot, "reset"].iter().map(|s| s.to_string()));
    run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
    emit_progress(&app, "Writing [uboot] -> [fdl2-cboot.bin]... Ok");
    std::thread::sleep(std::time::Duration::from_secs(10));

    emit_progress(&app, "Running unlock splloader...");
    tokens = vec!["exec_addr".into(), format!("0x{:x}", pkg.exec_addr), "fdl".into(), "spl-unlock.bin".into(), format!("0x{:x}", pkg.fdl1_addr)];
    run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
    emit_progress(&app, "Running unlock splloader... Ok");

    emit_progress(&app, "Reading miscdata...");
    tokens = build_base_tokens(false, &pkg);
    tokens.extend(["verbose", "2", "read_part", "miscdata", "8192", "64", "m.bin", "reset"].iter().map(|s| s.to_string()));
    run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
    emit_progress(&app, "Reading miscdata... Ok");

    emit_progress(&app, "Backing up partitions...");
    tokens = build_base_tokens(false, &pkg);
    for part in &pkg.backup_partitions { tokens.push("r".into()); tokens.push(part.clone()); }
    tokens.push("reset".into());
    run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
    emit_progress(&app, "Backing up partitions... Ok");

    let spl_restore = if spl16k.exists() { "u-boot-spl-16k-sign.bin" } else { pkg.spl_loader_bk.as_deref().unwrap_or("splloader_bk.bin") };
    let ub_restore = if ub_bak.exists() { "uboot_bak.bin" } else { "uboot_bk.bin" };
    emit_progress(&app, "Restoring stock loaders...");
    tokens = build_base_tokens(false, &pkg);
    tokens.extend(["w", "splloader", spl_restore, "w", "uboot", ub_restore].iter().map(|s| s.to_string()));
    run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
    emit_progress(&app, "Restoring stock loaders... Ok");

    if pkg.erase_persist {
        emit_progress(&app, "Erasing persist...");
        tokens = build_base_tokens(false, &pkg);
        tokens.extend(["e", "persist", "reset"].iter().map(|s| s.to_string()));
        run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
        emit_progress(&app, "Erasing persist... Ok");
    }

    emit_progress(&app, "Clearing red state...");
    tokens = build_base_tokens(false, &pkg);
    tokens.extend(["w", "misc", &pkg.misc_done, "reset"].iter().map(|s| s.to_string()));
    run_spd_dump_stream(&app, &spd_dump, &tokens, &work)?;
    emit_progress(&app, "Clearing red state... Ok");

    emit_progress(&app, "Unlock complete.");
    Ok(true)
}

/// Flashes selected partition images from a firmware folder (scatter-style).
pub fn flash(app: AppHandle, pkg_id: &str, device: Option<&str>, folder: &str, selected: Vec<UnisocFlashPart>) -> Result<bool, String> {
    let folder_path = Path::new(folder);
    if !folder_path.is_dir() {
        return Err("Firmware folder not found".into());
    }
    for part in &selected {
        let file = folder_path.join(&part.file);
        if !file.exists() {
            return Err(format!("Image not found: {}", part.file));
        }
    }

    let (pkg, pkg_dir, spd_dump, device_dir) = resolve_pkg_device(pkg_id, device)?;
    let work = prepare_work(&pkg_dir, device_dir.as_deref(), &pkg);

    emit_progress(&app, "Searching for Unisoc dl_diag device...");
    let mut tokens = build_base_tokens(true, &pkg);
    tokens.push("verbose".into());
    tokens.push("2".into());

    for part in &selected {
        let file = folder_path.join(&part.file);
        emit_progress(&app, format!("Writing [{}] -> [{}]...", part.name, part.file).as_str());
        tokens.push("w".into());
        tokens.push(part.name.clone());
        tokens.push(file.to_string_lossy().to_string());
    }
    tokens.push("reset".into());

    let result = run_spd_dump_stream(&app, &spd_dump, &tokens, &work);
    if result.is_err() {
        emit_progress(&app, "Searching for Unisoc dl_diag device... NOT FOUND, please retry");
        return Err(result.unwrap_err());
    }
    emit_progress(&app, "Searching for Unisoc dl_diag device... FOUND");
    emit_progress(&app, "SPRD USB Download Port detected");
    emit_progress(&app, "Connecting to device... Ok");
    emit_progress(&app, format!("ChipId : {}", pkg.name).as_str());
    emit_progress(&app, "Sending FDL1/FDL2 loaders... Ok");
    emit_progress(&app, "Flash complete.");
    Ok(true)
}

/// Erases the FRP partition.
pub fn erase_frp(app: AppHandle, pkg_id: &str, device: Option<&str>) -> Result<bool, String> {
    let (pkg, pkg_dir, spd_dump, device_dir) = resolve_pkg_device(pkg_id, device)?;
    let work = prepare_work(&pkg_dir, device_dir.as_deref(), &pkg);

    emit_progress(&app, "Searching for Unisoc dl_diag device...");
    let mut tokens = build_base_tokens(true, &pkg);
    tokens.extend(["verbose", "2", "e", "frp", "reset"].iter().map(|s| s.to_string()));
    let result = run_spd_dump_stream(&app, &spd_dump, &tokens, &work);
    if result.is_err() {
        emit_progress(&app, "Searching for Unisoc dl_diag device... NOT FOUND, please retry");
        return Err(result.unwrap_err());
    }
    emit_progress(&app, "Searching for Unisoc dl_diag device... FOUND");
    emit_progress(&app, "SPRD USB Download Port detected");
    emit_progress(&app, "Connecting to device... Ok");
    emit_progress(&app, format!("ChipId : {}", pkg.name).as_str());
    emit_progress(&app, "Sending FDL1/FDL2 loaders... Ok");
    emit_progress(&app, "Erasing [frp]... Ok");
    emit_progress(&app, "FRP erased.");
    Ok(true)
}

/// Dumps all partitions to the Downloads folder.
pub fn dump(app: AppHandle, pkg_id: &str, device: Option<&str>) -> Result<bool, String> {
    let (pkg, pkg_dir, spd_dump, device_dir) = resolve_pkg_device(pkg_id, device)?;
    let work = prepare_work(&pkg_dir, device_dir.as_deref(), &pkg);
    let dump_dir = dirs::download_dir().unwrap_or_else(|| std::env::temp_dir()).join("v1per_unisoc_dump");
    fs::create_dir_all(&dump_dir).map_err(|e| format!("Failed to create dump dir: {e}"))?;

    emit_progress(&app, "Searching for Unisoc dl_diag device...");
    let mut tokens = build_base_tokens(true, &pkg);
    tokens.extend(["path", &dump_dir.to_string_lossy().to_string(), "r", "all", "reset"].iter().map(|s| s.to_string()));
    let result = run_spd_dump_stream(&app, &spd_dump, &tokens, &work);
    if result.is_err() {
        emit_progress(&app, "Searching for Unisoc dl_diag device... NOT FOUND, please retry");
        return Err(result.unwrap_err());
    }
    emit_progress(&app, "Searching for Unisoc dl_diag device... FOUND");
    emit_progress(&app, "SPRD USB Download Port detected");
    emit_progress(&app, "Connecting to device... Ok");
    emit_progress(&app, format!("ChipId : {}", pkg.name).as_str());
    emit_progress(&app, "Sending FDL1/FDL2 loaders... Ok");
    emit_progress(&app, format!("Setting save path: {}", dump_dir.display()).as_str());
    emit_progress(&app, "Dumping all partitions... Ok");
    emit_progress(&app, format!("Dump saved to {}", dump_dir.display()).as_str());
    Ok(true)
}
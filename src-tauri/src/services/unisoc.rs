use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

static CURRENT_PROCESS: Mutex<Option<u32>> = Mutex::new(None);
static mut RESOURCE_DIR: Option<PathBuf> = None;

pub fn set_resource_dir(path: PathBuf) {
    unsafe { RESOURCE_DIR = Some(path); }
}

fn get_resource_dir() -> PathBuf {
    unsafe {
        RESOURCE_DIR.clone().unwrap_or_else(|| {
            // Fallback: walk up from exe
            let exe = std::env::current_exe().unwrap_or_default();
            exe.parent().unwrap_or(&std::path::PathBuf::from(".")).to_path_buf()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn get_packages() -> HashMap<String, UnisocPackage> {
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
    // Check resource_dir/Unisoc (Tauri bundled resources)
    let candidate = resource_dir.join("Unisoc");
    if candidate.exists() { return Some(candidate); }
    // Check resource_dir/resources/Unisoc
    let candidate = resource_dir.join("resources").join("Unisoc");
    if candidate.exists() { return Some(candidate); }
    // Fallback: walk up from exe
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent()?;
    for _ in 0..6 {
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
    if local.exists() { return Some(local) } else { None }
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

fn run_spd_dump(exe: &Path, tokens: &[String], cwd: &Path) -> Result<String, String> {
    let output = Command::new(exe)
        .args(tokens)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to run spd_dump: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_helper(pkg_dir: &Path, tools_gen: &str, exe: &str, arg: &str, cwd: &Path) -> Result<String, String> {
    let exe_name = if exe.ends_with(".exe") { exe.to_string() } else { format!("{}.exe", exe) };
    let mut exe_path = None;
    if let Some(root) = find_unisoc_root() {
        let shared = root.join("tools").join(tools_gen).join(&exe_name);
        if shared.exists() { exe_path = Some(shared); }
    }
    if exe_path.is_none() {
        let local = pkg_dir.join(&exe_name);
        if local.exists() { exe_path = Some(local); }
    }
    let path = exe_path.ok_or_else(|| format!("{} not found", exe_name))?;
    let output = Command::new(path)
        .arg(arg)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", exe_name, e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn get_packages() -> Result<HashMap<String, bool>, String> {
    let pkgs = get_packages();
    let root = find_unisoc_root();
    let mut result = HashMap::new();
    for (id, _) in &pkgs {
        let installed = root.as_ref().map(|r| r.join(id).exists()).unwrap_or(false);
        result.insert(id.clone(), installed);
    }
    Ok(result)
}

pub async fn stop_process() -> Result<bool, String> {
    let mut proc = CURRENT_PROCESS.lock().map_err(|e| e.to_string())?;
    if let Some(pid) = proc.take() {
        unsafe { libc::terminate_process(pid as _, 1); }
        return Ok(true);
    }
    Ok(false)
}

pub async fn unlock_bootloader(pkg_id: &str, device: Option<&str>) -> Result<bool, String> {
    let pkgs = get_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?;
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let device_dir = device.map(|d| pkg_dir.join(d)).filter(|d| d.exists());
    let work = prepare_work(&pkg_dir, device_dir.as_ref(), pkg);
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;

    // Step 1: Backup and erase
    let mut tokens = build_base_tokens(true, pkg);
    tokens.extend(["r", "splloader", "r", "uboot", "e", "splloader", "e", "splloader_bak", "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;

    // Generate spl-unlock
    let unlocker = work.join("spl-unlock.bin");
    if !unlocker.exists() {
        let spl_source = pkg.spl_loader_bk.as_deref().unwrap_or("splloader.bin");
        run_helper(&pkg_dir, &pkg.tools_gen, "gen_spl-unlock", spl_source, &work)?;
    }

    // Rename files
    let spl16k = work.join("u-boot-spl-16k-sign.bin");
    let spl_bin = work.join("splloader.bin");
    if spl_bin.exists() { fs::rename(&spl_bin, &spl16k).ok(); }
    if pkg.chsize_uboot {
        run_helper(&pkg_dir, &pkg.tools_gen, "chsize", "uboot.bin", &work).ok();
    }
    let ub_bin = work.join("uboot.bin");
    let ub_bak = work.join("uboot_bak.bin");
    if ub_bin.exists() { fs::rename(&ub_bin, &ub_bak).ok(); }

    // Step 2: Flash modified uboot
    tokens = build_base_tokens(true, pkg);
    tokens.extend(["w", "uboot", &pkg.cboot, "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Send unlocker
    tokens = vec![
        "exec_addr".into(), format!("0x{:x}", pkg.exec_addr),
        "fdl".into(), "spl-unlock.bin".into(), format!("0x{:x}", pkg.fdl1_addr),
    ];
    run_spd_dump(&spd_dump, &tokens, &work)?;

    // Check status
    tokens = build_base_tokens(false, pkg);
    tokens.extend(["verbose", "2", "read_part", "miscdata", "8192", "64", "m.bin", "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;

    // Backup partitions
    tokens = build_base_tokens(false, pkg);
    for part in &pkg.backup_partitions { tokens.push("r".into()); tokens.push(part.clone()); }
    tokens.push("reset".into());
    run_spd_dump(&spd_dump, &tokens, &work)?;

    // Step 3: Restore
    let spl_restore = if spl16k.exists() { "u-boot-spl-16k-sign.bin" }
        else { pkg.spl_loader_bk.as_deref().unwrap_or("splloader_bk.bin") };
    let ub_restore = if ub_bak.exists() { "uboot_bak.bin" } else { "uboot_bk.bin" };
    tokens = build_base_tokens(false, pkg);
    tokens.extend(["w", "splloader", spl_restore, "w", "uboot", ub_restore]);
    if pkg.erase_persist { tokens.extend(["e", "persist"]); }
    tokens.extend(["w", "misc", &pkg.misc_done, "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;

    Ok(true)
}

pub async fn dump_partitions(pkg_id: &str, device: Option<&str>) -> Result<bool, String> {
    let pkgs = get_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?;
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let device_dir = device.map(|d| pkg_dir.join(d)).filter(|d| d.exists());
    let work = prepare_work(&pkg_dir, device_dir.as_ref(), pkg);
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;
    let dump_dir = dirs::download_dir().unwrap_or_else(|| std::env::temp_dir()).join("v1per_unisoc_dump");
    fs::create_dir_all(&dump_dir).ok();
    let mut tokens = build_base_tokens(true, pkg);
    tokens.extend(["path", &dump_dir.to_string_lossy(), "r", "all", "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;
    Ok(true)
}

pub async fn flash_partition(pkg_id: &str, device: Option<&str>, partition: &str, image: &str) -> Result<bool, String> {
    if !Path::new(image).exists() { return Err("Image not found".into()); }
    let pkgs = get_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?;
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let device_dir = device.map(|d| pkg_dir.join(d)).filter(|d| d.exists());
    let work = prepare_work(&pkg_dir, device_dir.as_ref(), pkg);
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;
    let mut tokens = build_base_tokens(true, pkg);
    tokens.extend(["w", partition, image, "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;
    Ok(true)
}

pub async fn erase_partition(pkg_id: &str, device: Option<&str>, partition: &str) -> Result<bool, String> {
    let pkgs = get_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?;
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let device_dir = device.map(|d| pkg_dir.join(d)).filter(|d| d.exists());
    let work = prepare_work(&pkg_dir, device_dir.as_ref(), pkg);
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;
    let mut tokens = build_base_tokens(true, pkg);
    tokens.extend(["e", partition, "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;
    Ok(true)
}

pub async fn list_partitions(pkg_id: &str, device: Option<&str>) -> Result<String, String> {
    let pkgs = get_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?;
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let device_dir = device.map(|d| pkg_dir.join(d)).filter(|d| d.exists());
    let work = prepare_work(&pkg_dir, device_dir.as_ref(), pkg);
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;
    let list_file = work.join("partition_list.txt");
    let mut tokens = build_base_tokens(true, pkg);
    tokens.extend(["path", &work.to_string_lossy(), "partition_list", &list_file.to_string_lossy(), "p", "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;
    if list_file.exists() {
        fs::read_to_string(&list_file).map_err(|e| e.to_string())
    } else {
        Ok("No partition list produced".into())
    }
}

pub async fn erase_frp(pkg_id: &str, device: Option<&str>) -> Result<bool, String> {
    let pkgs = get_packages();
    let pkg = pkgs.get(pkg_id).ok_or("Unknown package")?;
    let root = find_unisoc_root().ok_or("Unisoc folder not found")?;
    let pkg_dir = root.join(pkg_id);
    if !pkg_dir.exists() { return Err("Package not installed".into()); }
    let device_dir = device.map(|d| pkg_dir.join(d)).filter(|d| d.exists());
    let work = prepare_work(&pkg_dir, device_dir.as_ref(), pkg);
    let spd_dump = resolve_spd_dump(&pkg_dir).ok_or("spd_dump.exe not found")?;
    let mut tokens = build_base_tokens(true, pkg);
    tokens.extend(["e", "frp", "reset"]);
    run_spd_dump(&spd_dump, &tokens, &work)?;
    Ok(true)
}

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use super::utils;

/// Emits one backup progress message to the renderer.
fn emit(app: &AppHandle, msg: &str) {
    let _ = app.emit("backup:progress", serde_json::json!({ "message": msg }));
}

/// Root folder that holds every backup card (folder per card).
/// Stored directly in the user's Downloads folder so the raw .img files are
/// always accessible in Explorer, while the GUI reads the same metadata.
fn backups_root() -> PathBuf {
    dirs::download_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("v1per-user-backups")
}

/// A card shown in the UI grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCard {
    pub name: String,
    pub model: String,
    pub date: String,
    pub partitions: usize,
    pub files: Vec<PartitionFile>,
}

/// One partition image inside a backup card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionFile {
    pub name: String,
    pub size: u64,
}

fn card_dir(name: &str) -> PathBuf {
    backups_root().join(name)
}

fn meta_path(name: &str) -> PathBuf {
    card_dir(name).join("backup.json")
}

fn now_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

// Howard Hinnant's civil_from_days algorithm (public domain).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Lists all saved backup cards.
pub fn list() -> Vec<BackupCard> {
    let root = backups_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut cards = Vec::new();
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(card) = read_meta(&name) {
            cards.push(card);
        }
    }
    cards
}

fn read_meta(name: &str) -> Option<BackupCard> {
    let data = fs::read_to_string(meta_path(name)).ok()?;
    let mut card: BackupCard = serde_json::from_str(&data).ok()?;
    // Always reflect what is actually on disk so removed/grown files show up.
    let dir = card_dir(name);
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "img").unwrap_or(false) {
            if let Ok(meta) = fs::metadata(&path) {
                files.push(PartitionFile {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    size: meta.len(),
                });
            }
        }
    }
    card.files = files;
    card.partitions = card.files.len();
    Some(card)
}

fn write_meta(card: &BackupCard) -> std::io::Result<()> {
    fs::create_dir_all(card_dir(&card.name))?;
    fs::write(meta_path(&card.name), serde_json::to_vec_pretty(card)?)
}

/// Renames a backup card folder (and its metadata).
pub fn rename(old: &str, new: &str) -> Result<(), String> {
    if old == new {
        return Ok(());
    }
    let new = new.trim().to_string();
    if new.is_empty() {
        return Err("Backup name cannot be empty.".into());
    }
    let from = card_dir(old);
    let to = card_dir(&new);
    if !from.exists() {
        return Err(format!("Backup '{old}' not found."));
    }
    if to.exists() {
        return Err(format!("Backup '{new}' already exists."));
    }
    fs::rename(&from, &to).map_err(|e| format!("Failed to rename: {e}"))?;
    if let Some(mut card) = read_meta(&new) {
        card.name = new.clone();
        let _ = write_meta(&card);
    }
    Ok(())
}

/// Deletes a backup card folder.
pub fn delete(name: &str) -> Result<(), String> {
    let dir = card_dir(name);
    if !dir.exists() {
        return Err(format!("Backup '{name}' not found."));
    }
    fs::remove_dir_all(&dir).map_err(|e| format!("Failed to delete: {e}"))
}

/// Copies the selected partition images of a card into a destination folder.
pub fn download_selected(name: &str, selected: Vec<String>, dest: &str) -> Result<usize, String> {
    let dest = PathBuf::from(dest);
    if dest.is_file() {
        return Err("Destination is a file, not a folder.".into());
    }
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create destination: {e}"))?;
    let src_dir = card_dir(name);
    if !src_dir.exists() {
        return Err(format!("Backup '{name}' not found."));
    }
    let mut copied = 0usize;
    for file in selected {
        let from = src_dir.join(&file);
        if !from.exists() {
            continue;
        }
        fs::copy(&from, dest.join(&file)).map_err(|e| format!("Failed to copy {file}: {e}"))?;
        copied += 1;
    }
    Ok(copied)
}

/// Runs a full partition backup for the given card name. Streams progress to
/// the renderer. Detects the device mode (ADB preferred, else FASTBOOT).
pub fn start(app: AppHandle, name: String) -> Result<bool, String> {
    let card = BackupCard {
        name,
        model: String::new(),
        date: now_str(),
        partitions: 0,
        files: Vec::new(),
    };
    fs::create_dir_all(card_dir(&card.name))
        .map_err(|e| format!("Failed to create backup folder: {e}"))?;

    let result = run_backup(&app, &card.name);
    match result {
        Ok((model, files)) => {
            let mut card = read_meta(&card.name).unwrap_or(card);
            card.model = model;
            card.partitions = files.len();
            card.files = files;
            let _ = write_meta(&card);
            emit(&app, "Backup complete -> saved to toolkit backups folder.");
            Ok(true)
        }
        Err(e) => {
            let _ = write_meta(&card);
            Err(e)
        }
    }
}

fn run_backup(app: &AppHandle, name: &str) -> Result<(String, Vec<PartitionFile>), String> {
    let started = std::time::Instant::now();

    emit(app, "Detecting device before backing up....");
    let (mode, serial) = utils::detect_mode();
    match mode.as_str() {
        "adb" => emit(app, "Detecting device before backing up.... ADB"),
        "fastboot" => emit(app, "Detecting device before backing up.... FASTBOOT"),
        _ => {
            emit(app, "Detecting device before backing up.... NOT FOUND");
            return Err("No device detected. Connect a device over USB and retry.".into());
        }
    }

    let serial = serial.ok_or("Device serial missing.")?;

    if mode == "adb" {
        emit(app, "Checking root access...");
        let id = utils::adb_serial(&serial, &["shell", "su", "-c", "id"], 8000);
        if !id.contains("uid=0") {
            emit(app, "Checking root access... NOT FOUND");
            return Err("Root access not granted. The device must be rooted and su must grant permission.".into());
        }
        emit(app, "Checking root access... FOUND");

        let model = utils::device_prop(&serial, "ro.product.model");
        emit(app, &format!("Device: {}", model));

        emit(app, "Checking boot slot...");
        let slot = utils::device_prop(&serial, "ro.boot.slot_suffix");
        emit(app, &format!("Checking boot slot... {slot}"));

        let partitions = utils::adb_serial(
            &serial,
            &["shell", "su", "-c", "ls /dev/block/by-name/"],
            10000,
        );
        let names: Vec<String> = partitions
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.contains('/'))
            .collect();
        if names.is_empty() {
            return Err("No partitions found under /dev/block/by-name/.".into());
        }
        emit(app, &format!("Found {} partitions.", names.len()));

        let mut files = Vec::new();
        let dest_dir = card_dir(name);
        for part in &names {
            emit(app, &format!("Backing up partition {}...", part));
            // Names come straight from /dev/block/by-name/ and already include
            // the slot suffix on A/B devices, so never append {slot} here.
            let src = format!("/dev/block/by-name/{part}");
            let dest = dest_dir.join(format!("{part}.img"));
            match utils::dump_partition(&serial, &src, &dest) {
                Ok(size) => {
                    emit(app, &format!("Backing up partition {}... DONE", part));
                    files.push(PartitionFile { name: format!("{part}.img"), size });
                }
                Err(e) => emit(app, &format!("Backing up partition {}... SKIPPED ({e})", part)),
            }
        }
        emit(app, &format!("Elapsed Time: {}s", started.elapsed().as_secs()).as_str());
        Ok((model, files))
    } else {
        emit(app, "Fastboot mode does not support raw partition dump over fastboot. Boot to system with root and retry.");
        Err("Partition backup requires ADB + root. Fastboot mode cannot dump partitions.".into())
    }
}
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use penumbra_mtk::da::{BootMode, ScatterFile};
use penumbra_mtk::hacc::LockState;
use penumbra_mtk::port::{ConnectionType, PortBackend, PortType};
use penumbra_mtk::{Device, DeviceBuilder, Storage};
use serde::Deserialize;
use tauri::{AppHandle, Emitter};

static SESSION: Mutex<Option<Device<'static, PortType>>> = Mutex::new(None);

fn emit(app: &AppHandle, status: &str, message: String) {
    let _ = app.emit("mtk:progress", serde_json::json!({ "status": status, "message": message }));
}

fn leaked(bytes: Vec<u8>) -> &'static [u8] {
    Box::leak(bytes.into_boxed_slice())
}

fn conn_type_str(conn: ConnectionType) -> &'static str {
    match conn {
        ConnectionType::Brom => "BROM",
        ConnectionType::Preloader => "Preloader",
        ConnectionType::Da => "DA",
    }
}

fn chip_name(device: &Device<'static, PortType>) -> String {
    let hw = device.devinfo().hw_code();
    device
        .devinfo()
        .chip()
        .map(|chip| format!("{chip:?}"))
        .unwrap_or_else(|| format!("Unknown (hw 0x{hw:04X})"))
}

fn noop_progress(_written: usize, _total: usize) {}

fn device_mut() -> Result<std::sync::MutexGuard<'static, Option<Device<'static, PortType>>>, String> {
    SESSION.lock().map_err(|_| "Session lock poisoned".to_string())
}

fn device_ref() -> Result<std::sync::MutexGuard<'static, Option<Device<'static, PortType>>>, String> {
    let session = SESSION.lock().map_err(|_| "Session lock poisoned".to_string())?;
    if session.is_none() {
        return Err("No device connected. Connect first.".to_string());
    }
    Ok(session)
}

/// Scans for a MediaTek device in BROM, Preloader or DA mode.
pub fn find_mtk_port() -> bool {
    PortType::find_device(Some(0x0E8D), None, PortBackend::Auto)
        .map(|port| port.is_some())
        .unwrap_or(false)
}

/// Connects to the device, waiting for it to appear, then performs the preloader/BROM
/// handshake and optionally loads a Download Agent to enter DA mode.
pub fn connect(
    app: AppHandle,
    da_path: Option<String>,
    auth_path: Option<String>,
    timeout_secs: u32,
) -> Result<serde_json::Value, String> {
    *SESSION.lock().unwrap() = None;

    let timeout = Duration::from_secs(timeout_secs.clamp(1, 600) as u64);
    let poll_interval = Duration::from_millis(250);
    let deadline = Instant::now() + timeout;

    emit(
        &app,
        "waiting",
        format!("Waiting up to {}s for a MediaTek device. Put it in BROM/Preloader mode.", timeout.as_secs()),
    );

    let port = loop {
        match PortType::find_device(Some(0x0E8D), None, PortBackend::Auto) {
            Ok(Some(port)) => break port,
            Ok(None) => {}
            Err(e) => {
                emit(&app, "waiting", format!("Scan failed ({e}), retrying..."));
            }
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "Timed out after {}s waiting for a MediaTek device. Connect the device in BROM or Preloader mode and try again.",
                timeout.as_secs()
            ));
        }

        std::thread::sleep(poll_interval);
    };

    emit(&app, "found", "MediaTek device detected. Starting handshake...".to_string());

    let da_data = match da_path {
        Some(path) => Some(leaked(std::fs::read(&path).map_err(|e| format!("Failed to read DA file: {e}"))?)),
        None => None,
    };
    let auth_data = match auth_path {
        Some(path) => Some(leaked(std::fs::read(&path).map_err(|e| format!("Failed to read auth file: {e}"))?)),
        None => None,
    };

    let builder = DeviceBuilder::new(port);
    let builder = if let Some(da) = da_data { builder.with_da_data(da) } else { builder };
    let builder = if let Some(auth) = auth_data { builder.with_auth(auth) } else { builder };
    let builder = builder.with_usb_log_channel(true);

    let mut device = builder.build().map_err(|e| format!("Failed to build device: {e}"))?;
    device.init().map_err(|e| format!("Handshake failed: {e}"))?;

    emit(&app, "handshake", "Handshake complete.".to_string());

    let connection = conn_type_str(device.get_connection_type()).to_string();
    let hw_code = device.devinfo().hw_code();
    let hw_subcode = device.devinfo().hw_subcode();
    let chip = chip_name(&device);

    let mut da_loaded = false;
    let mut partitions: Vec<String> = Vec::new();
    if da_data.is_some() {
        emit(&app, "da", "Loading Download Agent...".to_string());
        device.enter_da_mode().map_err(|e| format!("Failed to load DA: {e}"))?;
        da_loaded = true;
        partitions = device.partitions().iter().map(|p| p.name.clone()).collect();
    }

    *SESSION.lock().unwrap() = Some(device);

    emit(&app, "done", if da_loaded { "DA loaded.".to_string() } else { "Connected (preloader mode).".to_string() });

    Ok(serde_json::json!({
        "connected": true,
        "connection": connection,
        "chip": chip,
        "hw_code": hw_code,
        "hw_subcode": hw_subcode,
        "da_loaded": da_loaded,
        "partitions": partitions,
    }))
}

/// Returns info about the current connection.
pub fn device_info() -> Result<serde_json::Value, String> {
    let session = device_ref()?;
    let device = session.as_ref().ok_or("No device connected. Connect first.")?;
    Ok(serde_json::json!({
        "connected": true,
        "connection": conn_type_str(device.get_connection_type()),
        "chip": chip_name(device),
        "hw_code": device.devinfo().hw_code(),
        "hw_subcode": device.devinfo().hw_subcode(),
    }))
}

/// Lists the partition table from the connected device.
pub fn list_partitions() -> Result<serde_json::Value, String> {
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    let parts: Vec<serde_json::Value> = device
        .partitions()
        .iter()
        .map(|p| serde_json::json!({ "name": p.name, "size": p.size, "address": p.address }))
        .collect();
    Ok(serde_json::json!({ "partitions": parts }))
}

/// Writes an image file to a partition (SP Flash Tool style, works on locked bootloaders).
pub fn write_partition(partition: &str, path: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open image: {e}"))?;
    let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
    let mut reader = BufReader::new(file);
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device
        .write_partition(partition, size, &mut reader, noop_progress)
        .map_err(|e| format!("Failed to write {partition}: {e}"))
}

/// Reads a partition to a file.
pub fn read_partition(partition: &str, path: &str) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
    let mut writer = BufWriter::new(file);
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device
        .read_partition(partition, &mut writer, noop_progress)
        .map_err(|e| format!("Failed to read {partition}: {e}"))
}

/// Erases a partition on the device.
pub fn erase_partition(partition: &str) -> Result<(), String> {
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device
        .erase_flash(partition, noop_progress)
        .map_err(|e| format!("Failed to erase {partition}: {e}"))
}

/// Reboots the device into the requested boot mode.
pub fn reboot(mode: &str) -> Result<(), String> {
    let boot_mode = match mode {
        "fastboot" => BootMode::Fastboot,
        "test" => BootMode::Test,
        "meta" => BootMode::Meta,
        _ => BootMode::Normal,
    };
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device.reboot(boot_mode).map_err(|e| format!("Failed to reboot: {e}"))
}

/// Reads the active boot slot from the Boot Control partition.
pub fn bootctrl() -> Result<serde_json::Value, String> {
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    let bootctrl = device.get_bootctrl().map_err(|e| format!("Failed to read boot control: {e}"))?;
    Ok(serde_json::json!({ "active_slot": format!("{:?}", bootctrl.get_active_slot()) }))
}

/// Returns the device storage type.
pub fn storage() -> Result<serde_json::Value, String> {
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    let kind = device.get_storage().map(|s| s.as_str().to_string()).unwrap_or_else(|| "Unknown".to_string());
    Ok(serde_json::json!({ "storage": kind }))
}

/// Sets the seccfg lock state (bootloader unlock via exploit).
pub fn set_seccfg_lock_state(unlock: bool) -> Result<(), String> {
    let state = if unlock { LockState::Unlock } else { LockState::Lock };
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device
        .set_seccfg_lock_state(state)
        .map_err(|e| format!("Failed to set lock state: {e}"))
}

/// Dumps efuses to a file.
pub fn read_efuses(path: &str) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
    let mut writer = BufWriter::new(file);
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device.read_efuses(&mut writer).map_err(|e| format!("Failed to read efuses: {e}"))
}

/// Writes efuses from a file. DANGEROUS - only use known-good data.
pub fn write_efuses(path: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
    let mut reader = BufReader::new(file);
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device.write_efuses(&mut reader, size).map_err(|e| format!("Failed to write efuses: {e}"))
}

/// Reads a 32-bit register value.
pub fn read_register(addr: u64) -> Result<serde_json::Value, String> {
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    let value = device.read_register(addr).map_err(|e| format!("Failed to read register: {e}"))?;
    Ok(serde_json::json!({ "address": addr, "value": value }))
}

/// Writes a 32-bit register value.
pub fn write_register(addr: u64, value: u32) -> Result<(), String> {
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device.write_register(addr, value).map_err(|e| format!("Failed to write register: {e}"))
}

/// Reads memory at an address into a file.
pub fn peek(addr: u64, size: usize, path: &str) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
    let mut writer = BufWriter::new(file);
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device
        .peek(addr, size, &mut writer, noop_progress)
        .map_err(|e| format!("Failed to peek memory: {e}"))
}

/// Writes memory at an address from a file.
pub fn poke(addr: u64, path: &str) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
    let mut reader = BufReader::new(file);
    let mut session = device_mut()?;
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    device
        .poke(addr, size, &mut reader, noop_progress)
        .map_err(|e| format!("Failed to poke memory: {e}"))
}

/// Powers down the device and closes the connection.
pub fn disconnect() -> Result<bool, String> {
    let mut session = SESSION.lock().map_err(|_| "Session lock poisoned".to_string())?;
    if let Some(mut device) = session.take() {
        let _ = device.shutdown();
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlashOptions {
    pub da_path: Option<String>,
    pub scatter_path: String,
    pub auth_path: Option<String>,
    pub force_brom: bool,
    pub use_preloader_from_fw: bool,
    pub force_brom_erase_preloader: bool,
    pub timeout_secs: Option<u32>,
}

fn parse_scatter(content: &str) -> Result<ScatterFile, String> {
    let trimmed = content.trim_start();
    if trimmed.starts_with('<') || content.contains("<?xml") {
        ScatterFile::from_xml(content).map_err(|e| format!("Invalid scatter XML: {e}"))
    } else {
        ScatterFile::from_yaml(content).map_err(|e| format!("Invalid scatter file: {e}"))
    }
}

fn find_preloader(dir: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with("preloader") && name.ends_with(".bin") {
            return Some(entry.path());
        }
    }
    None
}

/// Parses a scatter file and returns the partition table (name, address, size, filename).
pub fn load_scatter(path: &str) -> Result<serde_json::Value, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read scatter file: {e}"))?;
    let scatter = parse_scatter(&content)?;

    let parts: Vec<serde_json::Value> = scatter
        .parts()
        .iter()
        .map(|p| {
            let filename = p
                .path
                .as_ref()
                .and_then(|f| f.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "NONE".to_string());
            serde_json::json!({
                "name": p.part.name,
                "address": format!("0x{:X}", p.part.address),
                "size": p.part.size,
                "filename": filename,
                "download": p.download,
            })
        })
        .collect();

    Ok(serde_json::json!({ "partitions": parts }))
}

/// Connects to the device and flashes all downloadable partitions from the scatter file.
pub fn flash(app: AppHandle, opts: FlashOptions) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    *SESSION.lock().unwrap() = None;

    let timeout = Duration::from_secs(opts.timeout_secs.unwrap_or(120).clamp(1, 600) as u64);
    let deadline = Instant::now() + timeout;

    emit(&app, "log", "Searching for usb device...".to_string());

    let port = loop {
        let res = if opts.force_brom {
            PortType::find_device(Some(0x0E8D), Some(0x0003), PortBackend::Auto)
        } else {
            PortType::find_device(Some(0x0E8D), None, PortBackend::Auto)
        };
        match res {
            Ok(Some(p)) => break p,
            Ok(None) => {}
            Err(e) => emit(&app, "log", format!("Scan failed ({e}), retrying...")),
        }
        if Instant::now() >= deadline {
            return Err(
                "Timed out waiting for a MediaTek device. Put the device in BROM or Preloader mode."
                    .to_string(),
            );
        }
        std::thread::sleep(Duration::from_millis(250));
    };

    emit(&app, "log", "Searching for usb device... OK".to_string());

    let scatter_content = std::fs::read_to_string(&opts.scatter_path)
        .map_err(|e| format!("Failed to read scatter file: {e}"))?;

    let da_data: Option<Vec<u8>> = if opts.use_preloader_from_fw {
        let base = Path::new(&opts.scatter_path)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        find_preloader(base)
            .map(|p| {
                std::fs::read(&p).map_err(|e| format!("Failed to read preloader from firmware: {e}"))
            })
            .transpose()?
    } else {
        opts.da_path
            .as_ref()
            .map(|p| std::fs::read(p).map_err(|e| format!("Failed to read DA file: {e}")))
            .transpose()?
    };

    let da_data = da_data.map(leaked);
    let auth_data = opts
        .auth_path
        .as_ref()
        .map(|p| leaked(std::fs::read(p).map_err(|e| format!("Failed to read auth file: {e}"))?));

    let builder = DeviceBuilder::new(port);
    let builder = if let Some(da) = da_data { builder.with_da_data(da) } else { builder };
    let builder = if let Some(auth) = auth_data { builder.with_auth(auth) } else { builder };
    let builder = builder.with_usb_log_channel(true);

    let mut device = builder.build().map_err(|e| format!("Failed to build device: {e}"))?;
    device.init().map_err(|e| format!("Handshake failed: {e}"))?;

    emit(&app, "log", "Connecting to device... OK".to_string());

    let chip = chip_name(&device);
    emit(&app, "log", format!("ChipId: {chip}"));

    device.enter_da_mode().map_err(|e| format!("Failed to load DA: {e}"))?;
    emit(&app, "log", "Sending Download-Agent... OK".to_string());

    emit(&app, "log", "Authorizing device for operation... OK".to_string());
    emit(&app, "log", "Reading partition information... OK".to_string());
    emit(&app, "log", "Reading system information... OK".to_string());

    if opts.force_brom_erase_preloader {
        emit(&app, "log", "Erasing preloader...".to_string());
        device
            .erase_flash("preloader", noop_progress)
            .map_err(|e| format!("Failed to erase preloader: {e}"))?;
        emit(&app, "log", "Erasing preloader... OK".to_string());
    }

    emit(&app, "log", "Rebuilding partition table... OK".to_string());

    let base = Path::new(&opts.scatter_path).parent().unwrap_or_else(|| Path::new(""));
    let scatter = parse_scatter(&scatter_content)?;
    let mut flashed: Vec<String> = Vec::new();
    let mut total_bytes = 0usize;

    for part in scatter.parts() {
        if !part.download {
            continue;
        }
        let Some(p) = &part.path else { continue };
        let full = base.join(p);
        let file = File::open(&full).map_err(|e| format!("Failed to open {}: {e}", full.display()))?;
        let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
        let file_name = p
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        emit(&app, "log", format!("Writing [{}] -> [{}]... ", part.part.name, file_name));
        let mut reader = BufReader::new(file);
        device
            .write_partition(&part.part.name, size, &mut reader, noop_progress)
            .map_err(|e| format!("Failed to write {}: {e}", part.part.name))?;
        emit(&app, "log", format!("Writing [{}] -> [{}]... OK", part.part.name, file_name));

        flashed.push(part.part.name.clone());
        total_bytes += size;
    }

    *SESSION.lock().unwrap() = Some(device);

    let mut session = SESSION.lock().map_err(|_| "Session lock poisoned".to_string())?;
    if let Some(mut d) = session.take() {
        let _ = d.reboot(BootMode::Normal);
    }
    drop(session);

    emit(&app, "log", "Rebooting device... OK".to_string());

    let elapsed = started.elapsed().as_secs();
    emit(&app, "log", format!("Elapsed time: {elapsed} Seconds"));

    Ok(serde_json::json!({
        "ok": true,
        "flashed": flashed,
        "bytes": total_bytes,
        "elapsed_secs": elapsed,
        "chip": chip,
    }))
}
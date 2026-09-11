use std::fs;
use std::sync::Mutex;

use penumbra_mtk::port::{ConnectionType, PortBackend, PortType};
use penumbra_mtk::{Device, DeviceBuilder};

static SESSION: Mutex<Option<Device<'static, PortType>>> = Mutex::new(None);

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

/// Scans for a MediaTek device in BROM, Preloader or DA mode.
pub fn find_mtk_port() -> bool {
    PortType::find_device(Some(0x0E8D), None, PortBackend::Auto)
        .map(|port| port.is_some())
        .unwrap_or(false)
}

/// Connects to the device, performs the preloader/BROM handshake and optionally
/// loads a Download Agent to enter DA mode.
pub fn connect(da_path: Option<String>, auth_path: Option<String>) -> Result<serde_json::Value, String> {
    *SESSION.lock().unwrap() = None;

    let port = PortType::find_device(Some(0x0E8D), None, PortBackend::Auto)
        .map_err(|e| format!("Failed to scan for MediaTek device: {e}"))?
        .ok_or("No MediaTek device found. Connect a device in BROM or Preloader mode.")?;

    let da_data = match da_path {
        Some(path) => Some(leaked(fs::read(&path).map_err(|e| format!("Failed to read DA file: {e}"))?)),
        None => None,
    };
    let auth_data = match auth_path {
        Some(path) => Some(leaked(fs::read(&path).map_err(|e| format!("Failed to read auth file: {e}"))?)),
        None => None,
    };

    let builder = DeviceBuilder::new(port);
    let builder = if let Some(da) = da_data { builder.with_da_data(da) } else { builder };
    let builder = if let Some(auth) = auth_data { builder.with_auth(auth) } else { builder };
    let builder = builder.with_usb_log_channel(true);

    let mut device = builder.build().map_err(|e| format!("Failed to build device: {e}"))?;
    device.init().map_err(|e| format!("Handshake failed: {e}"))?;

    let connection = conn_type_str(device.get_connection_type()).to_string();
    let hw_code = device.devinfo().hw_code();
    let hw_subcode = device.devinfo().hw_subcode();
    let chip = chip_name(&device);

    let mut da_loaded = false;
    let mut partitions: Vec<String> = Vec::new();
    if da_data.is_some() {
        device.enter_da_mode().map_err(|e| format!("Failed to load DA: {e}"))?;
        da_loaded = true;
        partitions = device.partitions().iter().map(|p| p.name.clone()).collect();
    }

    *SESSION.lock().unwrap() = Some(device);

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
    let session = SESSION.lock().unwrap();
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
    let mut session = SESSION.lock().unwrap();
    let device = session.as_mut().ok_or("No device connected. Connect first.")?;
    let parts: Vec<serde_json::Value> = device
        .partitions()
        .iter()
        .map(|p| serde_json::json!({ "name": p.name, "size": p.size, "address": p.address }))
        .collect();
    Ok(serde_json::json!({ "partitions": parts }))
}

/// Powers down the device and closes the connection.
pub fn disconnect() -> Result<bool, String> {
    let mut session = SESSION.lock().unwrap();
    if let Some(mut device) = session.take() {
        let _ = device.shutdown();
        Ok(true)
    } else {
        Ok(false)
    }
}
pub mod device;
pub mod unisoc;

use serde::{Deserialize, Serialize};
use tauri::command;

// ── Re-export device commands ────────────────────────────────

#[command]
pub async fn detect_device() -> Result<super::services::device::DetectResult, String> {
    let (mode, serial) = super::services::adb::detect().await;
    Ok(super::services::device::DetectResult { mode, serial })
}

#[command]
pub async fn check_adb() -> Result<bool, String> {
    Ok(super::services::device::check_adb().await)
}

#[command]
pub async fn check_fastboot() -> Result<bool, String> {
    Ok(super::services::device::check_fastboot().await)
}

#[command]
pub async fn run_adb_command(args: Vec<String>) -> Result<String, String> {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    Ok(super::services::adb::run_adb(&arg_refs).await)
}

#[command]
pub async fn run_fastboot_command(args: Vec<String>) -> Result<String, String> {
    Ok(super::services::fastboot::run_command(args).await)
}

#[command]
pub async fn get_device_info() -> Result<String, String> {
    Ok(super::services::device::get_info().await)
}

#[command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[command]
pub async fn open_path(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())
}

// ── Re-export unisoc commands ────────────────────────────────

#[command]
pub async fn get_packages() -> Result<std::collections::HashMap<String, bool>, String> {
    super::services::unisoc::get_packages().await
}

#[command]
pub async fn stop_process() -> Result<bool, String> {
    super::services::unisoc::stop_process().await
}

#[command]
pub async fn unlock_bootloader(pkg_id: String, device: Option<String>) -> Result<bool, String> {
    super::services::unisoc::unlock_bootloader(&pkg_id, device.as_deref()).await
}

#[command]
pub async fn dump_partitions(pkg_id: String, device: Option<String>) -> Result<bool, String> {
    super::services::unisoc::dump_partitions(&pkg_id, device.as_deref()).await
}

#[command]
pub async fn flash_partition(pkg_id: String, device: Option<String>, partition: String, image: String) -> Result<bool, String> {
    super::services::unisoc::flash_partition(&pkg_id, device.as_deref(), &partition, &image).await
}

#[command]
pub async fn erase_partition(pkg_id: String, device: Option<String>, partition: String) -> Result<bool, String> {
    super::services::unisoc::erase_partition(&pkg_id, device.as_deref(), &partition).await
}

#[command]
pub async fn list_partitions(pkg_id: String, device: Option<String>) -> Result<String, String> {
    super::services::unisoc::list_partitions(&pkg_id, device.as_deref()).await
}

#[command]
pub async fn erase_frp(pkg_id: String, device: Option<String>) -> Result<bool, String> {
    super::services::unisoc::erase_frp(&pkg_id, device.as_deref()).await
}

#[command]
pub async fn select_file(title: String, filters: Vec<String>) -> Result<Option<String>, String> {
    let result = rfd::FileDialog::new()
        .set_title(&title)
        .add_filter("Files", &filters)
        .pick_file();
    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

#[command]
pub async fn confirm_action(message: String) -> Result<bool, String> {
    let result = rfd::MessageDialog::new()
        .set_title("Confirm Action")
        .set_description(&message)
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show();
    Ok(result == rfd::MessageDialogResult::Ok)
}

use tauri::command;

// ── Device Commands ──────────────────────────────────────────

#[command]
pub async fn detect_device() -> Result<crate::services::device::DetectResult, String> {
    Ok(crate::services::device::detect().await)
}

#[command]
pub async fn check_adb() -> Result<bool, String> {
    Ok(crate::services::device::check_adb().await)
}

#[command]
pub async fn check_fastboot() -> Result<bool, String> {
    Ok(crate::services::device::check_fastboot().await)
}

#[command]
pub async fn run_adb_command(args: Vec<String>) -> Result<String, String> {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    Ok(crate::services::adb::run_adb(&arg_refs).await)
}

#[command]
pub async fn run_fastboot_command(args: Vec<String>) -> Result<String, String> {
    Ok(crate::services::fastboot::run_command(args).await)
}

#[command]
pub async fn get_device_info() -> Result<String, String> {
    Ok(crate::services::device::get_info().await)
}

#[command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[command]
pub async fn open_path(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())
}

// ── MediaTek Commands ─────────────────────────────────────────

#[command]
pub fn mtk_find_port() -> bool {
    crate::services::mtk::find_mtk_port()
}

#[command]
pub fn mtk_load_scatter(path: String) -> Result<serde_json::Value, String> {
    crate::services::mtk::load_scatter(&path)
}

#[command]
pub async fn mtk_flash(
    app: tauri::AppHandle,
    opts: crate::services::mtk::FlashOptions,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || crate::services::mtk::flash(app, opts))
        .await
        .map_err(|e| format!("Flash task failed: {e}"))?
}

#[command]
pub async fn mtk_connect(
    app: tauri::AppHandle,
    da_path: Option<String>,
    auth_path: Option<String>,
    timeout_secs: Option<u32>,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::mtk::connect(app, da_path, auth_path, timeout_secs.unwrap_or(120))
    })
    .await
    .map_err(|e| format!("Connect task failed: {e}"))?
}

#[command]
pub fn mtk_device_info() -> Result<serde_json::Value, String> {
    crate::services::mtk::device_info()
}

#[command]
pub fn mtk_list_partitions() -> Result<serde_json::Value, String> {
    crate::services::mtk::list_partitions()
}

#[command]
pub fn mtk_disconnect() -> Result<bool, String> {
    crate::services::mtk::disconnect()
}

#[command]
pub fn mtk_write_partition(partition: String, path: String) -> Result<(), String> {
    crate::services::mtk::write_partition(&partition, &path)
}

#[command]
pub fn mtk_read_partition(partition: String, path: String) -> Result<(), String> {
    crate::services::mtk::read_partition(&partition, &path)
}

#[command]
pub fn mtk_erase_partition(partition: String) -> Result<(), String> {
    crate::services::mtk::erase_partition(&partition)
}

#[command]
pub fn mtk_reboot(mode: String) -> Result<(), String> {
    crate::services::mtk::reboot(&mode)
}

#[command]
pub fn mtk_bootctrl() -> Result<serde_json::Value, String> {
    crate::services::mtk::bootctrl()
}

#[command]
pub fn mtk_storage() -> Result<serde_json::Value, String> {
    crate::services::mtk::storage()
}

#[command]
pub fn mtk_set_seccfg_lock_state(unlock: bool) -> Result<(), String> {
    crate::services::mtk::set_seccfg_lock_state(unlock)
}

#[command]
pub fn mtk_read_efuses(path: String) -> Result<(), String> {
    crate::services::mtk::read_efuses(&path)
}

#[command]
pub fn mtk_write_efuses(path: String) -> Result<(), String> {
    crate::services::mtk::write_efuses(&path)
}

#[command]
pub fn mtk_read_register(addr: u64) -> Result<serde_json::Value, String> {
    crate::services::mtk::read_register(addr)
}

#[command]
pub fn mtk_write_register(addr: u64, value: u32) -> Result<(), String> {
    crate::services::mtk::write_register(addr, value)
}

#[command]
pub fn mtk_peek(addr: u64, size: u64, path: String) -> Result<(), String> {
    crate::services::mtk::peek(addr, size as usize, &path)
}

#[command]
pub fn mtk_poke(addr: u64, path: String) -> Result<(), String> {
    crate::services::mtk::poke(addr, &path)
}

// ── Unisoc Commands ──────────────────────────────────────────

#[command]
pub fn get_packages() -> Result<std::collections::HashMap<String, bool>, String> {
    Ok(crate::services::unisoc::get_packages_installed())
}

#[command]
pub fn stop_process() -> Result<bool, String> {
    Ok(crate::services::unisoc::stop_process())
}

#[command]
pub fn unisoc_scan_folder(path: String) -> Result<serde_json::Value, String> {
    crate::services::unisoc::scan_folder(&path)
}

#[command]
pub async fn unisoc_unlock(
    app: tauri::AppHandle,
    pkg_id: String,
    device: Option<String>,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::unisoc::unlock(app, &pkg_id, device.as_deref())
    })
    .await
    .map_err(|e| format!("Unlock task failed: {e}"))?
}

#[command]
pub async fn unisoc_flash(
    app: tauri::AppHandle,
    pkg_id: String,
    device: Option<String>,
    folder: String,
    partitions: Vec<crate::services::unisoc::UnisocFlashPart>,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::unisoc::flash(app, &pkg_id, device.as_deref(), &folder, partitions)
    })
    .await
    .map_err(|e| format!("Flash task failed: {e}"))?
}

#[command]
pub async fn unisoc_erase_frp(
    app: tauri::AppHandle,
    pkg_id: String,
    device: Option<String>,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::unisoc::erase_frp(app, &pkg_id, device.as_deref())
    })
    .await
    .map_err(|e| format!("Erase FRP task failed: {e}"))?
}

#[command]
pub async fn unisoc_dump(
    app: tauri::AppHandle,
    pkg_id: String,
    device: Option<String>,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::unisoc::dump(app, &pkg_id, device.as_deref())
    })
    .await
    .map_err(|e| format!("Dump task failed: {e}"))?
}

#[command]
pub fn select_file(title: String, filters: Vec<String>) -> Result<Option<String>, String> {
    let result = rfd::FileDialog::new()
        .set_title(&title)
        .add_filter("Files", &filters)
        .pick_file();
    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

#[command]
pub fn select_folder(title: String) -> Result<Option<String>, String> {
    let result = rfd::FileDialog::new().set_title(&title).pick_folder();
    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

#[command]
pub fn confirm_action(message: String) -> Result<bool, String> {
    let result = rfd::MessageDialog::new()
        .set_title("Confirm Action")
        .set_description(&message)
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show();
    Ok(result == rfd::MessageDialogResult::Ok)
}

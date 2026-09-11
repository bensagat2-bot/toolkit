mod commands;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Set resource directory for Unisoc tools
            if let Some(resource_dir) = app.path().resource_dir().ok() {
                services::unisoc::set_resource_dir(resource_dir);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_device,
            commands::mtk_find_port,
            commands::mtk_load_scatter,
            commands::mtk_flash,
            commands::mtk_connect,
            commands::mtk_device_info,
            commands::mtk_list_partitions,
            commands::mtk_disconnect,
            commands::mtk_write_partition,
            commands::mtk_read_partition,
            commands::mtk_erase_partition,
            commands::mtk_reboot,
            commands::mtk_bootctrl,
            commands::mtk_storage,
            commands::mtk_set_seccfg_lock_state,
            commands::mtk_read_efuses,
            commands::mtk_write_efuses,
            commands::mtk_read_register,
            commands::mtk_write_register,
            commands::mtk_peek,
            commands::mtk_poke,
            commands::get_packages,
            commands::stop_process,
            commands::unisoc_scan_folder,
            commands::unisoc_unlock,
            commands::unisoc_flash,
            commands::unisoc_erase_frp,
            commands::unisoc_dump,
            commands::select_file,
            commands::select_folder,
            commands::confirm_action,
            commands::check_adb,
            commands::check_fastboot,
            commands::run_adb_command,
            commands::run_fastboot_command,
            commands::get_device_info,
            commands::open_url,
            commands::open_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

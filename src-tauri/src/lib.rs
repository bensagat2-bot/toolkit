mod commands;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_device,
            commands::get_packages,
            commands::stop_process,
            commands::unlock_bootloader,
            commands::dump_partitions,
            commands::flash_partition,
            commands::erase_partition,
            commands::list_partitions,
            commands::erase_frp,
            commands::select_file,
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

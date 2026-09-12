mod commands;
mod protection;
mod resources;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    protection::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // platform-tools, scrcpy and Unisoc ship as bundle resources next to the exe.
            let bundle = resources::bundle_dir();
            services::unisoc::set_resource_dir(bundle.clone());
            services::utils::set_resource_dir(bundle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_hwid,
            commands::get_device_model,
            commands::ota_list_partitions,
            commands::ota_extract_partition,
            commands::ota_extract_tgz,
            commands::frbox_list_partitions,
            commands::frbox_extract_partition,
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
            commands::unisoc_run_cli,
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
            commands::driver_download,
            commands::utils_root,
            commands::utils_anykernel,
            commands::utils_force_fastboot,
            commands::utils_scrcpy,
            commands::xiaomi_detect_device,
            commands::xiaomi_get_product,
            commands::xiaomi_get_token,
            commands::xiaomi_fastboot_stage,
            commands::xiaomi_fastboot_oem_unlock,
            commands::xiaomi_reboot_bootloader,
            commands::xiaomi_scan_rom,
            commands::xiaomi_run_fastboot,
            commands::xiaomi_run_adb,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod commands;
mod config;
mod log;
mod protection;
mod resources;
mod services;

#[cfg(windows)]
fn ensure_admin() {
    extern "system" {
        fn IsUserAnAdmin() -> i32;
    }
    let is_admin = unsafe { IsUserAnAdmin() != 0 };
    if is_admin {
        log::write("Running as administrator: YES");
    } else {
        log::write("Running as administrator: NO");
    }
}

// Release bundled binaries (adb server, fastboot, scrcpy, spd_dump) when the
// app exits so an upgrade/reinstall can overwrite them without file-lock errors.
fn kill_locked_tools() {
    for exe in ["adb.exe", "fastboot.exe", "scrcpy.exe", "spd_dump.exe"] {
        let mut cmd = std::process::Command::new("taskkill");
        cmd.args(["/F", "/IM", exe]);
        #[cfg(windows)]
        {
            // No console window - taskkill is a console app and would flash.
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let _ = cmd.status();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    protection::init();
    log::init();
    log::write("Application started");

    #[cfg(windows)]
    ensure_admin();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // Prune tool caches left behind by older app versions.
            resources::prune_old_caches();
            // Legacy layout fallback: tools next to the exe (bundle resources).
            let bundle = resources::bundle_dir();
            services::unisoc::set_resource_dir(bundle.clone());
            services::utils::set_resource_dir(bundle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_hwid,
            commands::get_device_model,
            commands::get_pc_info,
            commands::ota_list_partitions,
            commands::ota_extract_partition,
            commands::ota_extract_tgz,
            commands::ota_list_fastboot_images,
            commands::ota_extract_fastboot_image,
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
            commands::term_run,
            commands::check_fastboot_driver,
            commands::ensure_su_access,
            commands::backup_list,
            commands::backup_start,
            commands::backup_rename,
            commands::backup_delete,
            commands::backup_download_selected,
            commands::debloat_list_packages,
            commands::debloat_uninstall,
            commands::debloat_enable,
            commands::debloat_get_icons,
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                kill_locked_tools();
            }
        });
}

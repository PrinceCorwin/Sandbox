mod commands;

use std::fs;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Open devtools in debug builds so we can see JS errors
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            // Create data directories on first launch
            fs::create_dir_all(data_dir.join("thumbnails")).ok();

            // Copy default config if none exists
            let config_path = data_dir.join("app_config.json");
            if !config_path.exists() {
                let resource_dir = app
                    .path()
                    .resource_dir()
                    .expect("failed to resolve resource dir");
                let default_config = resource_dir.join("app_config.json");
                if default_config.exists() {
                    fs::copy(&default_config, &config_path).ok();
                } else {
                    // Write minimal default
                    fs::write(
                        &config_path,
                        r#"{"tags":[],"tag_order":[],"apps":{}}"#,
                    )
                    .ok();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::save_full_config,
            commands::config::update_app,
            commands::config::mark_used,
            commands::config::add_tag,
            commands::config::delete_tag,
            commands::config::reorder_tags,
            commands::config::reorder_apps,
            commands::discovery::discover_apps,
            commands::thumbnails::save_thumbnail,
            commands::thumbnails::get_thumbnail_base64,
            commands::fw_allocation::run_fw_allocation,
            commands::datalore::export_sqlite,
            commands::fs_helpers::app_copy_file,
            commands::fs_helpers::app_copy_tree,
            commands::fs_helpers::app_open_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::fs;
use std::path::Path;

// Generic file-system helpers usable by any miniapp.
//
// All commands are prefixed `app_` to avoid colliding with names exposed by
// `tauri-plugin-fs` (e.g. its built-in `copy_file`). Tauri's invoke router
// matches plugin commands by bare name, so a custom command sharing a plugin
// name gets routed to the plugin instead and fails its permission check.

#[tauri::command]
pub fn app_copy_file(src: String, dst: String) -> Result<(), String> {
    // Save-as helper: copies a single file using the app's full process
    // permissions, sidestepping tauri-plugin-fs's path-scope rules so users
    // can save to Downloads, Desktop, network drives, etc.
    fs::copy(&src, &dst)
        .map(|_| ())
        .map_err(|e| format!("Copy failed: {}", e))
}

#[tauri::command]
pub fn app_copy_tree(src: String, dst: String) -> Result<(), String> {
    // Save-as helper for multi-file outputs. Recursively copies a directory
    // tree using std::fs (no plugin scope checks).
    copy_dir_recursive(Path::new(&src), Path::new(&dst))
        .map_err(|e| format!("Copy failed: {}", e))
}

#[tauri::command]
pub fn app_open_path(path: String) -> Result<(), String> {
    // Reveal-in-Explorer helper. Windows-only app per CLAUDE.md.
    // explorer.exe handles both files and folders.
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

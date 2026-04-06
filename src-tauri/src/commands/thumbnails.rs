use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn thumbnails_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("thumbnails")
}

#[tauri::command]
pub fn save_thumbnail(
    app: AppHandle,
    app_id: String,
    source_path: String,
) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let filename = format!("{}.{}", app_id, ext);
    let dest_dir = thumbnails_dir(&app);
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(&filename);

    fs::copy(&source, &dest).map_err(|e| e.to_string())?;

    // Update the config with the thumbnail reference
    super::config::update_app(
        app,
        app_id,
        serde_json::json!({"thumbnail": filename}),
    )?;

    Ok(filename)
}

#[tauri::command]
pub fn get_thumbnail_base64(app: AppHandle, filename: String) -> Result<String, String> {
    use std::io::Read;

    let path = thumbnails_dir(&app).join(&filename);
    if !path.exists() {
        return Err("Thumbnail not found".to_string());
    }

    let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let mime = match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:{};base64,{}", mime, b64))
}

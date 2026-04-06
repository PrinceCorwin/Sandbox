use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("app_config.json")
}

fn read_config(app: &AppHandle) -> Result<Value, String> {
    let path = config_path(app);
    if !path.exists() {
        return Ok(serde_json::json!({"tags": [], "tag_order": [], "apps": {}}));
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

fn write_config(app: &AppHandle, config: &Value) -> Result<(), String> {
    let path = config_path(app);
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Result<Value, String> {
    read_config(&app)
}

#[tauri::command]
pub fn save_full_config(app: AppHandle, config: Value) -> Result<(), String> {
    write_config(&app, &config)
}

#[tauri::command]
pub fn update_app(app: AppHandle, app_id: String, data: Value) -> Result<(), String> {
    let mut config = read_config(&app)?;

    let apps = config
        .as_object_mut()
        .ok_or("config is not an object")?
        .entry("apps")
        .or_insert_with(|| serde_json::json!({}));

    let app_entry = apps
        .as_object_mut()
        .ok_or("apps is not an object")?
        .entry(&app_id)
        .or_insert_with(|| serde_json::json!({}));

    if let (Some(entry), Some(updates)) = (app_entry.as_object_mut(), data.as_object()) {
        for (key, value) in updates {
            entry.insert(key.clone(), value.clone());
        }
    }

    write_config(&app, &config)
}

#[tauri::command]
pub fn mark_used(app: AppHandle, app_id: String) -> Result<(), String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    update_app(
        app,
        app_id,
        serde_json::json!({"last_used": today}),
    )
}

#[tauri::command]
pub fn add_tag(app: AppHandle, name: String) -> Result<Value, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Tag name required".to_string());
    }

    let mut config = read_config(&app)?;

    let tags = config
        .as_object_mut()
        .ok_or("config is not an object")?
        .entry("tags")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or("tags is not an array")?;

    if tags.iter().any(|t| t.as_str() == Some(&name)) {
        return Err("Tag already exists".to_string());
    }

    tags.push(Value::String(name.clone()));

    let tag_order = config
        .as_object_mut()
        .unwrap()
        .entry("tag_order")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or("tag_order is not an array")?;

    tag_order.push(Value::String(name));

    write_config(&app, &config)?;

    Ok(config
        .get("tags")
        .cloned()
        .unwrap_or(serde_json::json!([])))
}

#[tauri::command]
pub fn delete_tag(app: AppHandle, tag_name: String) -> Result<Value, String> {
    let mut config = read_config(&app)?;

    {
        let obj = config.as_object_mut().ok_or("config is not an object")?;

        // Remove from tags array
        if let Some(tags) = obj.get_mut("tags").and_then(|t| t.as_array_mut()) {
            tags.retain(|t| t.as_str() != Some(&tag_name));
        }

        // Remove from tag_order
        if let Some(order) = obj.get_mut("tag_order").and_then(|t| t.as_array_mut()) {
            order.retain(|t| t.as_str() != Some(&tag_name));
        }

        // Remove from all apps' tags
        if let Some(apps) = obj.get_mut("apps").and_then(|a| a.as_object_mut()) {
            for app_data in apps.values_mut() {
                if let Some(app_tags) = app_data.get_mut("tags").and_then(|t| t.as_array_mut()) {
                    app_tags.retain(|t| t.as_str() != Some(&tag_name));
                }
            }
        }
    }

    write_config(&app, &config)?;

    Ok(config
        .get("tags")
        .cloned()
        .unwrap_or(serde_json::json!([])))
}

#[tauri::command]
pub fn reorder_tags(app: AppHandle, order: Vec<String>) -> Result<(), String> {
    let mut config = read_config(&app)?;

    config
        .as_object_mut()
        .ok_or("config is not an object")?
        .insert(
            "tag_order".to_string(),
            serde_json::json!(order),
        );

    write_config(&app, &config)
}

#[tauri::command]
pub fn reorder_apps(app: AppHandle, order: Vec<String>) -> Result<(), String> {
    let mut config = read_config(&app)?;

    if let Some(apps) = config
        .as_object_mut()
        .and_then(|o| o.get_mut("apps"))
        .and_then(|a| a.as_object_mut())
    {
        for (idx, app_id) in order.iter().enumerate() {
            if let Some(app_data) = apps.get_mut(app_id).and_then(|a| a.as_object_mut()) {
                app_data.insert("order".to_string(), serde_json::json!(idx));
            }
        }
    }

    write_config(&app, &config)
}

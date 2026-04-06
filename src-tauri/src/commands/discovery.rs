use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub tags: Vec<String>,
    pub order: i64,
    pub favorite: bool,
    pub thumbnail: Option<String>,
    pub date_added: String,
    pub last_used: Option<String>,
    pub url: String,
}

#[tauri::command]
pub fn discover_apps(app: AppHandle) -> Result<Vec<AppInfo>, String> {
    eprintln!("[discover_apps] called");

    // Read user config
    let config = match super::config::get_config(app.clone()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[discover_apps] get_config error: {}", e);
            serde_json::json!({"tags": [], "tag_order": [], "apps": {}})
        }
    };
    let apps_config = config.get("apps").cloned().unwrap_or(serde_json::json!({}));

    // Scan for app.json files in the apps/ directory.
    let resource_dir = app.path().resource_dir().ok();
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cwd = std::env::current_dir().ok();

    eprintln!("[discover_apps] resource_dir: {:?}", resource_dir);
    eprintln!("[discover_apps] CARGO_MANIFEST_DIR: {:?}", manifest_dir);
    eprintln!("[discover_apps] cwd: {:?}", cwd);

    // Check dev paths first (they have actual app.json files),
    // then fall back to resource_dir for production builds.
    let possible_dirs: Vec<std::path::PathBuf> = vec![
        Some(manifest_dir.join("..").join("src").join("apps")),
        cwd.map(|p| p.join("src").join("apps")),
        resource_dir.map(|p| p.join("apps")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for dir in &possible_dirs {
        let has_app_json = dir.is_dir()
            && fs::read_dir(dir).ok().map_or(false, |mut entries| {
                entries.any(|e| {
                    e.ok()
                        .map_or(false, |e| e.path().join("app.json").exists())
                })
            });
        eprintln!("[discover_apps] checking: {:?} is_dir={} has_apps={}", dir, dir.is_dir(), has_app_json);
    }

    // Find first directory that actually contains miniapp folders with app.json
    let apps_dir = possible_dirs
        .into_iter()
        .find(|p| {
            p.is_dir()
                && fs::read_dir(p).ok().map_or(false, |mut entries| {
                    entries.any(|e| {
                        e.ok()
                            .map_or(false, |e| e.path().join("app.json").exists())
                    })
                })
        });

    let apps_dir = match apps_dir {
        Some(dir) => {
            eprintln!("[discover_apps] using: {:?}", dir);
            dir
        }
        None => {
            eprintln!("[discover_apps] NO apps dir found!");
            return Ok(vec![]);
        }
    };

    let mut result = Vec::new();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut entries: Vec<_> = fs::read_dir(&apps_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let app_json_path = entry.path().join("app.json");
        if !app_json_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&app_json_path).map_err(|e| e.to_string())?;
        let meta: AppMeta = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let saved = apps_config
            .get(&meta.id)
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let info = AppInfo {
            id: meta.id.clone(),
            title: get_str(&saved, "title").unwrap_or(meta.name),
            description: get_str(&saved, "description").unwrap_or(meta.description),
            icon: meta.icon,
            tags: get_string_array(&saved, "tags"),
            order: saved.get("order").and_then(|v| v.as_i64()).unwrap_or(999),
            favorite: saved.get("favorite").and_then(|v| v.as_bool()).unwrap_or(false),
            thumbnail: get_str(&saved, "thumbnail"),
            date_added: get_str(&saved, "date_added").unwrap_or(today.clone()),
            last_used: get_str(&saved, "last_used"),
            url: format!("apps/{}/index.html", meta.id),
        };
        result.push(info);
    }

    result.sort_by_key(|a| a.order);
    Ok(result)
}

fn get_str(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn get_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

use crate::utils::settings::{self, AppSettings};
use tauri::command;

/// Get config directory path
#[command]
pub async fn get_config_dir() -> Result<String, String> {
    let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;

    Ok(config_dir
        .to_str()
        .ok_or("Failed to convert path to string")?
        .to_string())
}

/// Get all settings
#[command]
pub async fn get_settings() -> Result<AppSettings, String> {
    settings::load_settings().map_err(|e| e.to_string())
}

/// Save all settings
#[command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    tracing::info!("⚙️ [SETTINGS] Saving application settings");
    let result = settings::save_settings(&settings).map_err(|e| e.to_string());

    if result.is_ok() {
        tracing::info!("✅ [SETTINGS] Settings saved successfully");
    }

    result
}

/// Update a single setting
#[command]
pub async fn update_setting(key: String, value: serde_json::Value) -> Result<AppSettings, String> {
    settings::update_setting(&key, value).map_err(|e| e.to_string())
}

/// Get theme setting
#[command]
pub async fn get_theme() -> Result<String, String> {
    let settings = settings::load_settings().map_err(|e| e.to_string())?;
    Ok(settings.theme)
}

/// Set theme setting
#[command]
pub async fn set_theme(theme: String) -> Result<(), String> {
    tracing::info!("🎨 [SETTINGS] Setting theme to: {}", theme);
    let value = serde_json::Value::String(theme.clone());
    settings::update_setting("theme", value).map_err(|e| e.to_string())?;

    tracing::info!("✅ [SETTINGS] Theme set to '{}'", theme);
    Ok(())
}

/// Open file path in explorer/file manager
#[command]
pub async fn open_path_in_explorer(path: String) -> Result<(), String> {
    use std::path::Path;

    let path_obj = Path::new(&path);
    let _parent_dir = path_obj.parent().unwrap_or_else(|| Path::new(&path));

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if path_obj.is_file() {
            // For files, open with /select flag to highlight the file
            Command::new("explorer")
                .args(&["/select,", path.as_str()])
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {}", e))?;
        } else {
            // For directories, just open the directory
            Command::new("explorer")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {}", e))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .args(&["-R", path.as_str()])
            .spawn()
            .map_err(|e| format!("Failed to open finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("xdg-open")
            .arg(_parent_dir.to_str().unwrap_or("."))
            .spawn();
    }

    tracing::info!("📂 [FILE] Opened path in explorer: {}", path);
    Ok(())
}

/// Delete a file
#[command]
pub async fn delete_file(path: String) -> Result<(), String> {
    use std::fs;

    fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;

    tracing::info!("🗑️ [FILE] File deleted: {}", path);
    Ok(())
}

/// Rename a file
#[command]
pub async fn rename_file(old_path: String, new_path: String) -> Result<(), String> {
    use std::fs;

    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename file: {}", e))?;

    tracing::info!("✏️ [FILE] File renamed from {} to {}", old_path, new_path);
    Ok(())
}

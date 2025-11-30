use crate::utils::settings::{self, AppSettings};
use tauri::command;

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

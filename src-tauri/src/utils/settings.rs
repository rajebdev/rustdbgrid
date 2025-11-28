use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Theme preference: "light", "dark", or "auto"
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Editor font size
    #[serde(default = "default_font_size")]
    pub editor_font_size: u32,

    /// Show line numbers in editor
    #[serde(default = "default_true")]
    pub editor_line_numbers: bool,

    /// Auto-complete enabled
    #[serde(default = "default_true")]
    pub editor_autocomplete: bool,

    /// Default page size for data grid
    #[serde(default = "default_page_size")]
    pub grid_page_size: u32,

    /// Show row numbers in grid
    #[serde(default = "default_true")]
    pub grid_row_numbers: bool,

    /// Confirm before delete
    #[serde(default = "default_true")]
    pub confirm_delete: bool,

    /// Auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,

    /// Last used connection ID
    #[serde(default)]
    pub last_connection_id: Option<String>,
}

fn default_theme() -> String {
    "auto".to_string()
}

fn default_font_size() -> u32 {
    13
}

fn default_page_size() -> u32 {
    100
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            editor_font_size: default_font_size(),
            editor_line_numbers: true,
            editor_autocomplete: true,
            grid_page_size: default_page_size(),
            grid_row_numbers: true,
            confirm_delete: true,
            auto_connect: false,
            last_connection_id: None,
        }
    }
}

/// Get the settings file path
pub fn get_settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = dirs::config_dir().ok_or("Could not find config directory")?;
    let app_dir = app_data.join("rustdbgrid");

    // Create directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir.join(SETTINGS_FILE))
}

/// Load settings from file
pub fn load_settings() -> Result<AppSettings, Box<dyn std::error::Error>> {
    let path = get_settings_path()?;

    if !path.exists() {
        // Return default settings if file doesn't exist
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(&path)?;
    let settings: AppSettings = serde_json::from_str(&content)?;

    Ok(settings)
}

/// Save settings to file
pub fn save_settings(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&path, content)?;

    Ok(())
}

/// Update a single setting
pub fn update_setting(
    key: &str,
    value: serde_json::Value,
) -> Result<AppSettings, Box<dyn std::error::Error>> {
    let mut settings = load_settings()?;

    match key {
        "theme" => {
            if let Some(v) = value.as_str() {
                settings.theme = v.to_string();
            }
        }
        "editor_font_size" => {
            if let Some(v) = value.as_u64() {
                settings.editor_font_size = v as u32;
            }
        }
        "editor_line_numbers" => {
            if let Some(v) = value.as_bool() {
                settings.editor_line_numbers = v;
            }
        }
        "editor_autocomplete" => {
            if let Some(v) = value.as_bool() {
                settings.editor_autocomplete = v;
            }
        }
        "grid_page_size" => {
            if let Some(v) = value.as_u64() {
                settings.grid_page_size = v as u32;
            }
        }
        "grid_row_numbers" => {
            if let Some(v) = value.as_bool() {
                settings.grid_row_numbers = v;
            }
        }
        "confirm_delete" => {
            if let Some(v) = value.as_bool() {
                settings.confirm_delete = v;
            }
        }
        "auto_connect" => {
            if let Some(v) = value.as_bool() {
                settings.auto_connect = v;
            }
        }
        "last_connection_id" => {
            settings.last_connection_id = value.as_str().map(|s| s.to_string());
        }
        _ => {
            return Err(format!("Unknown setting key: {}", key).into());
        }
    }

    save_settings(&settings)?;
    Ok(settings)
}

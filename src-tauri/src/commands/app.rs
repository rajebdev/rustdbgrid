#[tauri::command]
pub fn get_app_version() -> String {
    env!("APP_VERSION").to_string()
}

#[tauri::command]
pub fn get_app_year() -> String {
    env!("APP_YEAR").to_string()
}

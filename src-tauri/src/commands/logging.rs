use crate::utils::tracing_logger;

/// Frontend logging command
/// Receives pre-formatted log messages from frontend and writes to frontend.log
#[tauri::command]
pub fn log_from_frontend(message: String) {
    // Write to frontend-specific log file
    tracing_logger::write_frontend_log(&message);
}

/// Bridge (sidecar) logging command
/// Receives pre-formatted log messages from bridge sidecar and writes to bridge.log
#[tauri::command]
pub fn log_from_bridge(message: String) {
    // Write to bridge-specific log file
    tracing_logger::write_bridge_log(&message);
}

/// Get log file paths for diagnostics/debugging
#[tauri::command]
pub fn get_log_info() -> Result<Vec<(String, String)>, String> {
    tracing_logger::get_log_paths()
        .map(|paths| {
            paths
                .into_iter()
                .map(|(name, path)| (name, path.display().to_string()))
                .collect()
        })
        .map_err(|e| e.to_string())
}

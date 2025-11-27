/// Frontend logging command
/// Receives pre-formatted log messages from frontend and outputs to stdout

#[tauri::command]
pub fn log_from_frontend(_level: String, message: String) {
    // Message is already formatted by FE, just print directly
    println!("{}", message);
}

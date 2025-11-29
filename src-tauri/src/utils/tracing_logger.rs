use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Get log directory path (sama dengan folder settings/connections)
pub fn get_log_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = dirs::config_dir().ok_or("Could not find config directory")?;
    let app_dir = app_data.join("rustdbgrid").join("logs");

    // Create directory if it doesn't exist
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir)
}

/// Initialize tracing subscriber dengan file appenders
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_log_directory()?;

    // File appender untuk Rust logs (daily rotation)
    let rust_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .filename_prefix("rust")
        .max_log_files(7) // Keep 7 days
        .build(&log_dir)?;

    // File appender untuk Frontend logs
    let frontend_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .filename_prefix("frontend")
        .max_log_files(7)
        .build(&log_dir)?;

    // File appender untuk Bridge logs
    let bridge_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .filename_prefix("bridge")
        .max_log_files(7)
        .build(&log_dir)?;

    // Custom formatter yang sesuai dengan format yang sudah ada
    let format = fmt::format()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(true)
        .with_level(true);

    // Filter berdasarkan environment atau default ke INFO untuk production
    let filter = EnvFilter::try_from_default_env().or_else(|_| {
        #[cfg(debug_assertions)]
        {
            EnvFilter::try_new("debug")
        }
        #[cfg(not(debug_assertions))]
        {
            EnvFilter::try_new("info")
        }
    })?;

    // Layer untuk Rust logs (stdout + file)
    let rust_file_layer = fmt::layer()
        .with_writer(rust_appender)
        .with_ansi(false)
        .event_format(format.clone());

    // Layer untuk stdout (development)
    #[cfg(debug_assertions)]
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .event_format(format.clone());

    // Initialize subscriber
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(filter)
        .with(rust_file_layer)
        .with(stdout_layer)
        .init();

    #[cfg(not(debug_assertions))]
    tracing_subscriber::registry()
        .with(filter)
        .with(rust_file_layer)
        .init();

    tracing::info!("✅ [TRACING] Logging system initialized");
    tracing::info!("📁 [TRACING] Log directory: {}", log_dir.display());

    // Store appenders untuk digunakan oleh logging commands
    store_appenders(frontend_appender, bridge_appender);

    Ok(())
}

use once_cell::sync::OnceCell;
use std::sync::Mutex;

static FRONTEND_APPENDER: OnceCell<Mutex<RollingFileAppender>> = OnceCell::new();
static BRIDGE_APPENDER: OnceCell<Mutex<RollingFileAppender>> = OnceCell::new();

fn store_appenders(frontend: RollingFileAppender, bridge: RollingFileAppender) {
    let _ = FRONTEND_APPENDER.set(Mutex::new(frontend));
    let _ = BRIDGE_APPENDER.set(Mutex::new(bridge));
}

/// Write log langsung ke file frontend
pub fn write_frontend_log(message: &str) {
    if let Some(appender) = FRONTEND_APPENDER.get() {
        use std::io::Write;
        if let Ok(mut writer) = appender.lock() {
            let now = chrono::Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
            let formatted = format!("[{}][FE] {}\n", timestamp, message);
            let _ = writer.write_all(formatted.as_bytes());
            let _ = writer.flush();
        }
    }
}

/// Write log langsung ke file bridge
pub fn write_bridge_log(message: &str) {
    if let Some(appender) = BRIDGE_APPENDER.get() {
        use std::io::Write;
        if let Ok(mut writer) = appender.lock() {
            let now = chrono::Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
            let formatted = format!("[{}][BRIDGE] {}\n", timestamp, message);
            let _ = writer.write_all(formatted.as_bytes());
            let _ = writer.flush();
        }
    }
}

/// Get current log file paths untuk diagnostics
pub fn get_log_paths() -> Result<Vec<(String, PathBuf)>, Box<dyn std::error::Error>> {
    let log_dir = get_log_directory()?;

    Ok(vec![
        ("Rust".to_string(), log_dir.join("rust.log")),
        ("Frontend".to_string(), log_dir.join("frontend.log")),
        ("Bridge".to_string(), log_dir.join("bridge.log")),
    ])
}

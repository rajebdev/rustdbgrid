// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod utils;

use commands::{connection, export, logging, query, schema, settings};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(log::LevelFilter::Debug)
                .format(|out, message, record| {
                    let now = chrono::Local::now();
                    let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
                    let target = record.target();
                    let level = record.level();
                    out.finish(format_args!(
                        "[{}][{};{}][RUST][{}] {}",
                        timestamp,
                        target,
                        record.line().map(|l| l.to_string()).unwrap_or_default(),
                        level,
                        message
                    ))
                })
                .build(),
        )
        .manage(connection::ConnectionStore::new())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Only shutdown when the main window closes
                if window.label() == "main" {
                    log::info!("🛑 [APP] Main window closing, shutting down bridge...");
                    db::shutdown_bridge();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            connection::test_connection,
            connection::save_connection,
            connection::get_connections,
            connection::delete_connection,
            connection::get_storage_info,
            connection::connect_to_database,
            connection::disconnect_from_database,
            connection::is_database_connected,
            connection::get_connected_databases,
            query::execute_query,
            query::execute_query_with_filters,
            query::get_filter_values,
            schema::get_databases,
            schema::get_tables,
            schema::get_table_schema,
            schema::get_table_data,
            export::export_schema,
            export::export_data,
            export::copy_schema,
            export::copy_data,
            logging::log_from_frontend,
            settings::get_settings,
            settings::save_settings,
            settings::update_setting,
            settings::get_theme,
            settings::set_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

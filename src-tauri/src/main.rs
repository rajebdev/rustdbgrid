// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod utils;

use commands::{connection, export, query, schema};

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
                .build(),
        )
        .manage(connection::ConnectionStore::new())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

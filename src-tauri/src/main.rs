// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod utils;

use commands::{connection, export, logging, query, schema, settings};

fn main() {
    // Initialize tracing logger
    if let Err(e) = utils::tracing_logger::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(connection::ConnectionStore::new())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Only shutdown when the main window closes
                if window.label() == "main" {
                    tracing::info!("🛑 [APP] Main window closing, shutting down bridge...");
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
            query::save_query,
            query::load_queries,
            query::delete_query,
            query::save_auto_query,
            query::load_auto_query,
            query::auto_save_query_file,
            query::get_next_query_number,
            query::list_query_files,
            query::list_query_files_with_content,
            query::delete_query_file,
            schema::get_databases,
            schema::get_tables,
            schema::get_views,
            schema::get_indexes,
            schema::get_procedures,
            schema::get_triggers,
            schema::get_events,
            schema::get_table_schema,
            schema::get_table_relationships,
            schema::get_table_statistics,
            schema::get_table_data,
            schema::get_pg_constraints,
            schema::get_pg_foreign_keys,
            schema::get_pg_indexes,
            schema::get_pg_references,
            schema::get_pg_partitions,
            export::export_schema,
            export::export_data,
            export::copy_schema,
            export::copy_data,
            logging::log_from_frontend,
            logging::log_from_bridge,
            logging::get_log_info,
            settings::get_settings,
            settings::save_settings,
            settings::update_setting,
            settings::get_theme,
            settings::set_theme,
            settings::get_config_dir,
            settings::open_path_in_explorer,
            settings::delete_file,
            settings::rename_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

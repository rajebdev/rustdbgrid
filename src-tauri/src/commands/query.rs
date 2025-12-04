use crate::commands::connection::ConnectionStore;
use crate::models::query_result::*;
use futures::FutureExt;
use tauri::State;

#[tauri::command]
pub async fn get_filter_values(
    connection_id: String,
    query: String,
    column: String,
    search_query: Option<String>,
    _limit: Option<usize>, // Intentionally unused - we want all distinct values
    state: State<'_, ConnectionStore>,
) -> Result<FilterValuesResult, String> {
    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        let config = {
            let connections = state.connections.lock().unwrap();
            connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| format!("Connection '{}' not found", connection_id))?
                .clone()
        };
        state.pool.connect(config).await?;
    }

    // Get db_type from connection
    let db_type = {
        let connections = state.connections.lock().unwrap();
        connections
            .iter()
            .find(|c| c.id == connection_id)
            .map(|c| c.db_type.clone())
            .ok_or_else(|| format!("Connection '{}' not found", connection_id))?
    };

    use crate::models::connection::DatabaseType;

    // For Ignite with SCAN queries, we need to get all data and extract distinct values client-side
    if matches!(db_type, DatabaseType::Ignite) {
        // For SCAN queries, execute the query and extract distinct values
        let query_clone = query.clone();
        let column_clone = column.clone();
        let result = state
            .pool
            .with_connection(&connection_id, |conn| {
                async move { conn.execute_query(&query_clone).await }.boxed()
            })
            .await?;

        // Extract distinct values from result
        let mut values_set = std::collections::HashSet::new();
        for row in result.rows {
            if let Some(value) = row.get(&column_clone) {
                let value_str = match value {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(value).unwrap_or_default(),
                };

                // Apply search filter if provided
                if let Some(ref search) = search_query {
                    if value_str.to_lowercase().contains(&search.to_lowercase()) {
                        values_set.insert(value_str);
                    }
                } else {
                    values_set.insert(value_str);
                }
            }
        }

        let mut values: Vec<String> = values_set.into_iter().collect();
        values.sort();
        let total_count = values.len();

        return Ok(FilterValuesResult {
            values,
            total_count,
        });
    }

    // Extract table name from simple SELECT queries
    // Pattern: SELECT * FROM table_name [LIMIT ...]
    // Preserve database.table format for cross-database queries (e.g., apps_config.jns_config)
    let cleaned_query = query.trim().to_uppercase();

    let table_name = if cleaned_query.starts_with("SELECT") && cleaned_query.contains("FROM") {
        // Find FROM keyword
        if let Some(from_pos) = cleaned_query.find("FROM") {
            let after_from = &query[from_pos + 4..].trim();

            // Extract table name (stop at LIMIT, WHERE, ORDER BY, etc.)
            let end_markers = ["LIMIT", "WHERE", "ORDER BY", "GROUP BY", ";"];
            let mut table_end = after_from.len();

            for marker in &end_markers {
                if let Some(pos) = after_from.to_uppercase().find(marker) {
                    table_end = table_end.min(pos);
                }
            }

            let table = after_from[..table_end].trim().to_string();
            table
        } else {
            // Fallback to subquery if can't extract
            query.clone()
        }
    } else {
        // Complex query, use as subquery
        query.clone()
    };

    // Build query based on database type
    let filter_query = match db_type {
        DatabaseType::MSSQL => {
            // MSSQL uses brackets for column names and different case-insensitive syntax
            let col_escaped = format!("[{}]", column.replace("]", "]]"));
            let table_escaped = if table_name.contains("SELECT") || table_name.contains("(") {
                format!("({}) AS subquery", query)
            } else {
                // Handle database.schema.table format for MSSQL
                table_name.clone()
            };

            if let Some(search) = &search_query {
                let escaped_search = search.replace("'", "''");
                // MSSQL uses COLLATE for case-insensitive search
                format!(
                    "SELECT DISTINCT {} FROM {} WHERE {} LIKE '%{}%' COLLATE Latin1_General_CI_AI ORDER BY {}",
                    col_escaped, table_escaped, col_escaped, escaped_search, col_escaped
                )
            } else {
                format!(
                    "SELECT DISTINCT {} FROM {} ORDER BY {}",
                    col_escaped, table_escaped, col_escaped
                )
            }
        }
        _ => {
            // Standard SQL for MySQL, PostgreSQL, etc.
            if table_name.contains("SELECT") || table_name.contains("(") {
                // Complex query - use subquery
                if let Some(search) = &search_query {
                    let escaped_search = search.replace("'", "''").to_lowercase();
                    format!(
                        "SELECT DISTINCT {} FROM ({}) AS subquery WHERE LOWER({}) LIKE '%{}%' ORDER BY {}",
                        column, query, column, escaped_search, column
                    )
                } else {
                    format!(
                        "SELECT DISTINCT {} FROM ({}) AS subquery ORDER BY {}",
                        column, query, column
                    )
                }
            } else {
                // Simple query - directly from table
                if let Some(search) = &search_query {
                    let escaped_search = search.replace("'", "''").to_lowercase();
                    format!(
                        "SELECT DISTINCT {} FROM {} WHERE LOWER({}) LIKE '%{}%' ORDER BY {}",
                        column, table_name, column, escaped_search, column
                    )
                } else {
                    format!(
                        "SELECT DISTINCT {} FROM {} ORDER BY {}",
                        column, table_name, column
                    )
                }
            }
        }
    };

    let query_clone = filter_query.clone();
    let result = state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.execute_query(&query_clone).await }.boxed()
        })
        .await?;

    // Extract distinct values
    let mut values = Vec::new();
    for row in result.rows {
        if let Some(value) = row.get(&column) {
            let value_str = match value {
                serde_json::Value::Null => "NULL".to_string(),
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            values.push(value_str);
        }
    }

    let total_count = values.len();

    Ok(FilterValuesResult {
        values,
        total_count,
    })
}

#[tauri::command]
pub async fn save_query(
    title: String,
    content: String,
    description: String,
    connection_id: Option<String>,
    database_name: Option<String>,
) -> Result<String, String> {
    use crate::models::saved_query::SavedQuery;
    use std::fs;

    let query_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("rustdbgrid")
        .join("queries");

    // Create queries directory if it doesn't exist
    fs::create_dir_all(&query_dir)
        .map_err(|e| format!("Failed to create queries directory: {}", e))?;

    // Create new query
    let query = SavedQuery::new(title, content, description, connection_id, database_name);
    let query_id = query.id.clone();

    // Load existing queries
    let queries_file = query_dir.join("queries.json");
    let mut queries_data: Vec<SavedQuery> = if queries_file.exists() {
        let content = fs::read_to_string(&queries_file)
            .map_err(|e| format!("Failed to read queries file: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Add new query
    queries_data.insert(0, query);

    // Save to file
    let json = serde_json::to_string_pretty(&queries_data)
        .map_err(|e| format!("Failed to serialize queries: {}", e))?;
    fs::write(&queries_file, json).map_err(|e| format!("Failed to save queries file: {}", e))?;

    tracing::info!("✅ Query saved with id: {}", query_id);
    Ok(query_id)
}

#[tauri::command]
pub async fn load_queries() -> Result<Vec<crate::models::saved_query::SavedQuery>, String> {
    use std::fs;

    let query_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("rustdbgrid")
        .join("queries");

    // Create queries directory if it doesn't exist
    fs::create_dir_all(&query_dir)
        .map_err(|e| format!("Failed to create queries directory: {}", e))?;

    let queries_file = query_dir.join("queries.json");

    if !queries_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&queries_file)
        .map_err(|e| format!("Failed to read queries file: {}", e))?;

    let queries: Vec<crate::models::saved_query::SavedQuery> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse queries file: {}", e))?;

    Ok(queries)
}

#[tauri::command]
pub async fn delete_query(query_id: String) -> Result<(), String> {
    use std::fs;

    let query_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("rustdbgrid")
        .join("queries");

    let queries_file = query_dir.join("queries.json");

    if !queries_file.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&queries_file)
        .map_err(|e| format!("Failed to read queries file: {}", e))?;

    let mut queries: Vec<crate::models::saved_query::SavedQuery> =
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse queries file: {}", e))?;

    queries.retain(|q| q.id != query_id);

    let json = serde_json::to_string_pretty(&queries)
        .map_err(|e| format!("Failed to serialize queries: {}", e))?;
    fs::write(&queries_file, json).map_err(|e| format!("Failed to save queries file: {}", e))?;

    tracing::info!("✅ Query deleted: {}", query_id);
    Ok(())
}

#[tauri::command]
pub async fn save_auto_query(
    tab_id: String,
    query: String,
    connection_id: Option<String>,
    database_name: Option<String>,
) -> Result<(), String> {
    use crate::models::saved_query::AutoSaveQuery;
    use std::fs;
    use std::time::SystemTime;

    let query_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("rustdbgrid")
        .join("queries");

    // Create queries directory if it doesn't exist
    fs::create_dir_all(&query_dir)
        .map_err(|e| format!("Failed to create queries directory: {}", e))?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let auto_save = AutoSaveQuery {
        tab_id,
        query,
        connection_id,
        database_name,
        saved_at: now,
    };

    let autosave_file = query_dir.join(".autosave.json");
    let json = serde_json::to_string_pretty(&auto_save)
        .map_err(|e| format!("Failed to serialize auto-save: {}", e))?;
    fs::write(&autosave_file, json).map_err(|e| format!("Failed to save auto-save file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_auto_query() -> Result<Option<crate::models::saved_query::AutoSaveQuery>, String>
{
    use std::fs;

    let query_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("rustdbgrid")
        .join("queries");

    let autosave_file = query_dir.join(".autosave.json");

    if !autosave_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&autosave_file)
        .map_err(|e| format!("Failed to read auto-save file: {}", e))?;

    let auto_save: crate::models::saved_query::AutoSaveQuery = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse auto-save file: {}", e))?;

    Ok(Some(auto_save))
}

#[tauri::command]
pub async fn auto_save_query_file(
    file_path: String,
    content: String,
) -> Result<serde_json::Value, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    // Write the file
    fs::write(path, content).map_err(|e| format!("Failed to write query file: {}", e))?;

    tracing::info!("💾 [AUTO-SAVE] Query auto-saved to: {}", file_path);

    Ok(serde_json::json!({
        "path": file_path,
        "success": true
    }))
}

#[tauri::command]
pub async fn get_next_query_number() -> Result<usize, String> {
    use std::fs;

    let query_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("rustdbgrid")
        .join("queries");

    // Create queries directory if it doesn't exist
    if !query_dir.exists() {
        fs::create_dir_all(&query_dir)
            .map_err(|e| format!("Failed to create queries directory: {}", e))?;
        return Ok(1);
    }

    // Read directory and find all Query N.sql files
    let mut max_number = 0;

    if let Ok(entries) = fs::read_dir(&query_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        // Check if filename matches pattern "Query N.sql"
                        if file_name.starts_with("Query ") && file_name.ends_with(".sql") {
                            let number_str = file_name
                                .strip_prefix("Query ")
                                .and_then(|s| s.strip_suffix(".sql"))
                                .unwrap_or("0");

                            if let Ok(number) = number_str.parse::<usize>() {
                                max_number = max_number.max(number);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(max_number + 1)
}

#[derive(serde::Serialize)]
pub struct QueryFileInfo {
    name: String,
    path: String,
    created: Option<String>,
    modified: Option<String>,
}

#[tauri::command]
pub async fn list_query_files(folder_path: String) -> Result<Vec<QueryFileInfo>, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&folder_path);

    // Create queries directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create queries directory: {}", e))?;
        return Ok(vec![]);
    }

    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        // Only include .sql files
                        if file_name.ends_with(".sql") {
                            let file_path = entry.path();

                            let created = metadata.created().ok().and_then(|t| {
                                chrono::DateTime::<chrono::Utc>::from(t)
                                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                                    .into()
                            });

                            let modified = metadata.modified().ok().and_then(|t| {
                                chrono::DateTime::<chrono::Utc>::from(t)
                                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                                    .into()
                            });

                            files.push(QueryFileInfo {
                                name: file_name.to_string(),
                                path: file_path.to_string_lossy().to_string(),
                                created,
                                modified,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by modified time (newest first)
    files.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(files)
}

#[tauri::command]
pub async fn delete_query_file(file_path: String) -> Result<bool, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;

    tracing::info!("🗑️ [DELETE] Query file deleted: {}", file_path);

    Ok(true)
}

#[derive(serde::Serialize)]
pub struct QueryFileWithContent {
    id: String,
    title: String,
    content: String,
    description: String,
    file_path: String,
    created_at: String,
    last_modified: String,
    is_file: bool,
}

#[tauri::command]
pub async fn list_query_files_with_content(
    folder_path: String,
) -> Result<Vec<QueryFileWithContent>, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&folder_path);

    // Create queries directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create queries directory: {}", e))?;
        return Ok(vec![]);
    }

    let mut queries = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        // Only include .sql files
                        if file_name.ends_with(".sql") {
                            let file_path = entry.path();

                            // Read file content
                            match fs::read_to_string(&file_path) {
                                Ok(content) => {
                                    let created = metadata
                                        .created()
                                        .ok()
                                        .map(|t| {
                                            let datetime: chrono::DateTime<chrono::Utc> = t.into();
                                            datetime
                                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                                        })
                                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                                    let modified = metadata
                                        .modified()
                                        .ok()
                                        .map(|t| {
                                            let datetime: chrono::DateTime<chrono::Utc> = t.into();
                                            datetime
                                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                                        })
                                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                                    let path_str = file_path.to_string_lossy().to_string();

                                    queries.push(QueryFileWithContent {
                                        id: path_str.clone(),
                                        title: file_name.replace(".sql", ""),
                                        content,
                                        description: String::new(),
                                        file_path: path_str,
                                        created_at: created,
                                        last_modified: modified,
                                        is_file: true,
                                    });
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to read query file {}: {}",
                                        file_name,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by modified time (newest first)
    queries.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(queries)
}

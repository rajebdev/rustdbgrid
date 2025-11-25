use crate::models::schema::*;
use crate::models::connection::*;
use crate::models::query_result::*;

#[tauri::command]
pub async fn get_databases(config: ConnectionConfig) -> Result<Vec<Database>, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);
    
    conn.connect(&config).await.map_err(|e| e.to_string())?;
    let databases = conn.get_databases().await.map_err(|e| e.to_string())?;
    conn.disconnect().await.map_err(|e| e.to_string())?;
    
    Ok(databases)
}

#[tauri::command]
pub async fn get_tables(config: ConnectionConfig, database: String) -> Result<Vec<Table>, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);
    
    conn.connect(&config).await.map_err(|e| e.to_string())?;
    let tables = conn.get_tables(&database).await.map_err(|e| e.to_string())?;
    conn.disconnect().await.map_err(|e| e.to_string())?;
    
    Ok(tables)
}

#[tauri::command]
pub async fn get_table_schema(
    config: ConnectionConfig,
    database: String,
    table: String,
) -> Result<TableSchema, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);
    
    conn.connect(&config).await.map_err(|e| e.to_string())?;
    let schema = conn.get_table_schema(&database, &table).await.map_err(|e| e.to_string())?;
    conn.disconnect().await.map_err(|e| e.to_string())?;
    
    Ok(schema)
}

#[tauri::command]
pub async fn get_table_data(
    config: ConnectionConfig,
    database: String,
    table: String,
    limit: u32,
    offset: u32,
) -> Result<QueryResult, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);
    
    conn.connect(&config).await.map_err(|e| e.to_string())?;
    let data = conn.get_table_data(&database, &table, limit, offset).await.map_err(|e| e.to_string())?;
    conn.disconnect().await.map_err(|e| e.to_string())?;
    
    Ok(data)
}

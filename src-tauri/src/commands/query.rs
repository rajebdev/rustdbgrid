use crate::commands::connection::ConnectionStore;
use crate::models::connection::*;
use crate::models::query_result::*;
use futures::FutureExt;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn execute_query(
    config: ConnectionConfig,
    query: String,
    state: State<'_, ConnectionStore>,
) -> Result<QueryResult, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    let query_clone = query.clone();
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.execute_query(&query_clone).await }.boxed()
        })
        .await
}
#[tauri::command]
pub async fn execute_query_with_filters(
    config: ConnectionConfig,
    base_query: String,
    filters: Option<HashMap<String, serde_json::Value>>,
    sort_column: Option<String>,
    sort_direction: Option<String>,
    limit: Option<usize>,
    state: State<'_, ConnectionStore>,
) -> Result<QueryResult, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config.clone()).await?;
    }

    // For MongoDB and Redis, execute the original query without modification
    // These databases don't use SQL syntax
    use crate::models::connection::DatabaseType;
    if matches!(config.db_type, DatabaseType::MongoDB | DatabaseType::Redis) {
        let query_clone = base_query.clone();
        let mut result = state
            .pool
            .with_connection(&connection_id, |conn| {
                async move { conn.execute_query(&query_clone).await }.boxed()
            })
            .await?;

        // Keep original query for display
        result.final_query = Some(base_query);
        return Ok(result);
    }

    // For MSSQL, if query already has pagination (ROW_NUMBER or TOP), use it as-is
    // Frontend buildPaginatedQuery already handles MSSQL pagination correctly
    if matches!(config.db_type, DatabaseType::MSSQL)
        && (base_query.to_uppercase().contains("ROW_NUMBER()")
            || base_query.to_uppercase().contains("TOP ")
            || (filters.is_none() && sort_column.is_none()))
    {
        let query_clone = base_query.clone();
        let mut result = state
            .pool
            .with_connection(&connection_id, |conn| {
                async move { conn.execute_query(&query_clone).await }.boxed()
            })
            .await?;

        result.final_query = Some(base_query);
        return Ok(result);
    }

    // For SQL databases, proceed with query building
    // Extract table name from simple SELECT queries to avoid subquery
    // Preserve database.table format for cross-database queries
    let cleaned_query = base_query.trim().to_uppercase();

    let table_name = if cleaned_query.starts_with("SELECT") && cleaned_query.contains("FROM") {
        if let Some(from_pos) = cleaned_query.find("FROM") {
            let after_from = &base_query[from_pos + 4..].trim();

            let end_markers = ["LIMIT", "WHERE", "ORDER BY", "GROUP BY", ";"];
            let mut table_end = after_from.len();

            for marker in &end_markers {
                if let Some(pos) = after_from.to_uppercase().find(marker) {
                    table_end = table_end.min(pos);
                }
            }

            // Extract and preserve database.table format (e.g., apps_config.jns_config)
            let table = after_from[..table_end].trim().to_string();
            table
        } else {
            base_query.clone()
        }
    } else {
        base_query.clone()
    };

    // Build filtered query - use table directly if simple, otherwise use subquery
    let use_direct_table = !table_name.contains("SELECT") && !table_name.contains("(");

    let mut query = if use_direct_table {
        format!("SELECT * FROM {}", table_name)
    } else {
        format!("SELECT * FROM ({}) AS subquery", base_query)
    };

    // Add WHERE clause for filters
    if let Some(filter_map) = filters {
        if !filter_map.is_empty() {
            let mut where_clauses = Vec::new();
            for (column, value) in filter_map.iter() {
                // Check if value is array (from modal) or string (from text input)
                if let Some(arr) = value.as_array() {
                    // Array filter - use IN clause
                    if !arr.is_empty() {
                        let values: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();

                        if !values.is_empty() {
                            // Escape single quotes in values
                            let escaped_values: Vec<String> = values
                                .iter()
                                .map(|v| {
                                    if v == "NULL" {
                                        column.to_string()
                                    } else {
                                        format!("'{}'", v.replace("'", "''"))
                                    }
                                })
                                .collect();

                            // Handle NULL values separately
                            if values.contains(&"NULL".to_string()) {
                                let non_null_values: Vec<String> = values
                                    .iter()
                                    .filter(|v| v.as_str() != "NULL")
                                    .map(|v| format!("'{}'", v.replace("'", "''")))
                                    .collect();

                                if non_null_values.is_empty() {
                                    where_clauses.push(format!("{} IS NULL", column));
                                } else {
                                    where_clauses.push(format!(
                                        "({} IN ({}) OR {} IS NULL)",
                                        column,
                                        non_null_values.join(", "),
                                        column
                                    ));
                                }
                            } else {
                                where_clauses.push(format!(
                                    "{} IN ({})",
                                    column,
                                    escaped_values.join(", ")
                                ));
                            }
                        }
                    }
                } else if let Some(str_value) = value.as_str() {
                    // String filter - use LIKE clause with LOWER for case-insensitive search
                    if !str_value.is_empty() {
                        let escaped_value = str_value.replace("'", "''").to_lowercase();
                        where_clauses.push(format!("LOWER({}) LIKE '%{}%'", column, escaped_value));
                    }
                }
            }

            if !where_clauses.is_empty() {
                query.push_str(&format!(" WHERE {}", where_clauses.join(" AND ")));
            }
        }
    }

    // Add ORDER BY clause
    if let Some(col) = sort_column {
        let direction = sort_direction.unwrap_or_else(|| "ASC".to_string());
        query.push_str(&format!(" ORDER BY {} {}", col, direction));
    }

    // Add LIMIT/pagination clause based on database type
    if let Some(limit_val) = limit {
        match config.db_type {
            DatabaseType::MSSQL => {
                // SQL Server uses TOP (if no ORDER BY) or OFFSET-FETCH (with ORDER BY)
                if query.to_uppercase().contains("ORDER BY") {
                    // Has ORDER BY, use OFFSET-FETCH
                    query.push_str(&format!(
                        " OFFSET 0 ROWS FETCH NEXT {} ROWS ONLY",
                        limit_val
                    ));
                } else {
                    // No ORDER BY, use TOP by modifying SELECT
                    query = query.replace("SELECT", &format!("SELECT TOP {}", limit_val));
                }
            }
            _ => {
                // Standard SQL LIMIT for MySQL, PostgreSQL, SQLite, etc.
                query.push_str(&format!(" LIMIT {}", limit_val));
            }
        }
    }

    let query_clone = query.clone();
    let mut result = state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.execute_query(&query_clone).await }.boxed()
        })
        .await?;

    // Add the final query to result for display
    result.final_query = Some(query);

    Ok(result)
}

#[tauri::command]
pub async fn get_filter_values(
    config: ConnectionConfig,
    query: String,
    column: String,
    search_query: Option<String>,
    _limit: Option<usize>, // Intentionally unused - we want all distinct values
    state: State<'_, ConnectionStore>,
) -> Result<FilterValuesResult, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
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

    // Build query to get ALL distinct values directly from table
    let filter_query = if table_name.contains("SELECT") || table_name.contains("(") {
        // Complex query - use subquery
        if let Some(search) = search_query {
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
        if let Some(search) = search_query {
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

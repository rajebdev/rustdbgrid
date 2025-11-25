use crate::models::connection::*;
use crate::models::query_result::*;
use std::collections::HashMap;

#[tauri::command]
pub async fn execute_query(config: ConnectionConfig, query: String) -> Result<QueryResult, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);

    conn.connect(&config).await.map_err(|e| e.to_string())?;
    let result = conn
        .execute_query(&query)
        .await
        .map_err(|e| e.to_string())?;
    conn.disconnect().await.map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn execute_query_with_filters(
    config: ConnectionConfig,
    base_query: String,
    filters: Option<HashMap<String, Vec<String>>>,
    sort_column: Option<String>,
    sort_direction: Option<String>,
    limit: Option<usize>,
) -> Result<QueryResult, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);

    conn.connect(&config).await.map_err(|e| e.to_string())?;

    // Build filtered query
    let mut query = format!("SELECT * FROM ({}) AS subquery", base_query);

    // Add WHERE clause for filters
    if let Some(filter_map) = filters {
        if !filter_map.is_empty() {
            let mut where_clauses = Vec::new();
            for (column, values) in filter_map.iter() {
                if !values.is_empty() {
                    // Escape single quotes in values
                    let escaped_values: Vec<String> = values
                        .iter()
                        .map(|v| {
                            if v == "NULL" {
                                column.to_string() // For NULL values, we'll handle separately
                            } else {
                                format!("'{}'", v.replace("'", "''"))
                            }
                        })
                        .collect();

                    // Handle NULL values separately
                    if values.contains(&"NULL".to_string()) {
                        let non_null_values: Vec<String> = values
                            .iter()
                            .filter(|v| *v != "NULL")
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

    // Add LIMIT clause
    if let Some(limit_val) = limit {
        query.push_str(&format!(" LIMIT {}", limit_val));
    }

    let result = conn
        .execute_query(&query)
        .await
        .map_err(|e| e.to_string())?;

    conn.disconnect().await.map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn get_filter_values(
    config: ConnectionConfig,
    query: String,
    column: String,
    search_query: Option<String>,
    limit: Option<usize>,
) -> Result<FilterValuesResult, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);

    conn.connect(&config).await.map_err(|e| e.to_string())?;

    // Build query to get ALL distinct values (no LIMIT)
    let filter_query = if let Some(search) = search_query {
        // Escape single quotes in search query
        let escaped_search = search.replace("'", "''");
        format!(
            "SELECT DISTINCT {} FROM ({}) AS subquery WHERE CAST({} AS TEXT) LIKE '%{}%' ORDER BY {}",
            column, query, column, escaped_search, column
        )
    } else {
        format!(
            "SELECT DISTINCT {} FROM ({}) AS subquery ORDER BY {}",
            column, query, column
        )
    };

    let result = conn
        .execute_query(&filter_query)
        .await
        .map_err(|e| e.to_string())?;

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

    conn.disconnect().await.map_err(|e| e.to_string())?;

    Ok(FilterValuesResult {
        values,
        total_count,
    })
}

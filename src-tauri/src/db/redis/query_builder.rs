use crate::db::traits::{CRUDQueryBuilder, QueryBuilder};
use crate::models::save_request::EditedRow;
use crate::models::table_request::*;
use anyhow::Result;
use std::collections::HashMap;

pub struct RedisQueryBuilder;

impl QueryBuilder for RedisQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        Ok(format!("SCAN 0 MATCH {}:*", request.table))
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        identifier.to_string()
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        request.table.clone()
    }

    fn build_where_clause(&self, _filters: &[Filter]) -> Result<String> {
        Ok("".to_string())
    }

    fn build_order_by_clause(&self, _order_by: &[OrderBy]) -> String {
        "".to_string()
    }

    fn build_pagination_clause(&self, _limit: usize, _offset: usize) -> String {
        "".to_string()
    }
}

impl CRUDQueryBuilder for RedisQueryBuilder {
    fn build_insert_query(
        &self,
        table: &str,
        _schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String> {
        if row.is_empty() {
            anyhow::bail!("Cannot insert empty row");
        }

        // For Redis, generate HSET command
        // Format: HSET key field1 value1 field2 value2 ...
        let mut parts = vec![format!("HSET {}", table)];

        // Use table_schema column order to ensure consistent order
        for col in &table_schema.columns {
            if let Some(v) = row.get(&col.name) {
                parts.push(col.name.clone());
                parts.push(match v {
                    serde_json::Value::Null => "".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                });
            }
        }

        Ok(format!("{};", parts.join(" ")))
    }

    fn build_update_query(
        &self,
        table: &str,
        _schema: Option<&str>,
        edited_row: &EditedRow,
        _primary_keys: &[&String],
        table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String> {
        let updated_data = &edited_row.updated_data;

        if updated_data.is_empty() {
            anyhow::bail!("Cannot update with no columns");
        }

        // For Redis, generate HSET command for update
        let mut parts = vec![format!("HSET {}", table)];

        // Use table_schema column order to ensure consistent order
        for col in &table_schema.columns {
            if let Some(v) = updated_data.get(&col.name) {
                parts.push(col.name.clone());
                parts.push(match v {
                    serde_json::Value::Null => "".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                });
            }
        }

        Ok(format!("{};", parts.join(" ")))
    }

    fn build_delete_query(
        &self,
        table: &str,
        _schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        _primary_keys: &[&String],
    ) -> Result<String> {
        // For Redis, generate HDEL command
        // Format: HDEL key field1 field2 ...
        let mut parts = vec![format!("HDEL {}", table)];
        for k in row.keys() {
            parts.push(k.clone());
        }

        Ok(format!("{};", parts.join(" ")))
    }

    fn format_value(&self, val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Null => "".to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => val.to_string(),
        }
    }

    fn format_where_condition(&self, val: &serde_json::Value) -> String {
        self.format_value(val)
    }

    fn escape_sql_string(&self, s: &str) -> String {
        s.to_string()
    }
}

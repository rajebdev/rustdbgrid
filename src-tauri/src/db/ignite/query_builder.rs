use crate::db::mysql::query_builder::MySQLQueryBuilder;
use crate::db::traits::{CRUDQueryBuilder, QueryBuilder};
use crate::models::save_request::EditedRow;
use crate::models::table_request::*;
use anyhow::Result;
use std::collections::HashMap;

pub struct IgniteQueryBuilder;

impl QueryBuilder for IgniteQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let table = self.format_table_name(request);
        let mut query = format!("SELECT * FROM {}", table);

        if !request.filters.is_empty() {
            let where_clause = self.build_where_clause(&request.filters)?;
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        if !request.order_by.is_empty() {
            let order_clause = self.build_order_by_clause(&request.order_by);
            query.push_str(&format!(" ORDER BY {}", order_clause));
        }

        let pagination = self.build_pagination_clause(request.limit, request.offset);
        query.push_str(&format!(" {}", pagination));

        Ok(query)
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("\"{}\"", identifier.replace("\"", "\"\""))
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        let table = self.quote_identifier(&request.table);

        if let Some(schema) = &request.schema {
            format!("{}.{}", self.quote_identifier(schema), table)
        } else {
            table
        }
    }

    fn build_where_clause(&self, filters: &[Filter]) -> Result<String> {
        let builder = MySQLQueryBuilder;
        let conditions: Vec<String> = filters
            .iter()
            .map(|f| {
                let column = self.quote_identifier(&f.column);
                let mut filter_copy = f.clone();
                filter_copy.column = column;
                builder.build_filter_condition(&filter_copy)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(conditions.join(" AND "))
    }

    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String {
        order_by
            .iter()
            .map(|o| {
                let direction = match o.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{} {}", self.quote_identifier(&o.column), direction)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String {
        format!("LIMIT {} OFFSET {}", limit, offset)
    }
}

impl CRUDQueryBuilder for IgniteQueryBuilder {
    fn build_insert_query(
        &self,
        table: &str,
        schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String> {
        if row.is_empty() {
            anyhow::bail!("Cannot insert empty row");
        }

        // Get column order from table_schema to ensure consistent column order
        let columns: Vec<String> = table_schema
            .columns
            .iter()
            .filter_map(|col| {
                if row.contains_key(&col.name) {
                    Some(col.name.clone())
                } else {
                    None
                }
            })
            .collect();

        let columns_str = columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");

        let values_str = columns
            .iter()
            .map(|col| self.format_value(&row[col]))
            .collect::<Vec<_>>()
            .join(", ");

        let table_name = if let Some(s) = schema {
            format!(
                "{}.{}",
                self.quote_identifier(s),
                self.quote_identifier(table)
            )
        } else {
            self.quote_identifier(table).to_string()
        };

        Ok(format!(
            "INSERT INTO {} ({}) VALUES ({});",
            table_name, columns_str, values_str
        ))
    }

    fn build_update_query(
        &self,
        table: &str,
        schema: Option<&str>,
        edited_row: &EditedRow,
        primary_keys: &[&String],
        table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String> {
        let updated_data = &edited_row.updated_data;
        let original_data = &edited_row.original_data;

        if updated_data.is_empty() {
            anyhow::bail!("Cannot update with no columns");
        }

        // Build SET clause using table_schema column order (exclude primary keys)
        let set_parts: Result<Vec<String>> = table_schema
            .columns
            .iter()
            .filter(|col| {
                updated_data.contains_key(&col.name)
                    && !primary_keys
                        .iter()
                        .any(|pk| pk.as_str() == col.name.as_str())
            })
            .map(|col| {
                let val = &updated_data[&col.name];
                Ok(format!(
                    "{} = {}",
                    self.quote_identifier(&col.name),
                    self.format_value(val)
                ))
            })
            .collect();

        let set_clause = set_parts?.join(", ");

        if set_clause.is_empty() {
            anyhow::bail!("No columns to update (all are primary keys)");
        }

        // Build WHERE clause
        let where_clause = if !primary_keys.is_empty() {
            // Use primary keys
            primary_keys
                .iter()
                .map(|pk| {
                    let val = original_data.get(pk.as_str()).ok_or_else(|| {
                        anyhow::anyhow!("Primary key {} not found in original data", pk)
                    })?;
                    Ok(format!(
                        "{} {}",
                        self.quote_identifier(pk),
                        self.format_where_condition(val)
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(" AND ")
        } else {
            // Fallback: use all original columns
            original_data
                .iter()
                .map(|(col, val)| {
                    Ok(format!(
                        "{} {}",
                        self.quote_identifier(col),
                        self.format_where_condition(val)
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(" AND ")
        };

        if where_clause.is_empty() {
            anyhow::bail!("Cannot generate WHERE clause");
        }

        // Format table name with schema
        let table_name = if let Some(s) = schema {
            format!(
                "{}.{}",
                self.quote_identifier(s),
                self.quote_identifier(table)
            )
        } else {
            self.quote_identifier(table).to_string()
        };

        Ok(format!(
            "UPDATE {} SET {} WHERE {};",
            table_name, set_clause, where_clause
        ))
    }

    fn build_delete_query(
        &self,
        table: &str,
        schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        primary_keys: &[&String],
    ) -> Result<String> {
        // Build WHERE clause
        let where_clause = if !primary_keys.is_empty() {
            // Use primary keys
            primary_keys
                .iter()
                .map(|pk| {
                    let val = row
                        .get(pk.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Primary key {} not found in row", pk))?;
                    Ok(format!(
                        "{} {}",
                        self.quote_identifier(pk),
                        self.format_where_condition(val)
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(" AND ")
        } else {
            // Fallback: use all columns from row
            row.iter()
                .map(|(col, val)| {
                    Ok(format!(
                        "{} {}",
                        self.quote_identifier(col),
                        self.format_where_condition(val)
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(" AND ")
        };

        if where_clause.is_empty() {
            anyhow::bail!("Cannot generate WHERE clause for DELETE");
        }

        // Format table name with schema
        let table_name = if let Some(s) = schema {
            format!(
                "{}.{}",
                self.quote_identifier(s),
                self.quote_identifier(table)
            )
        } else {
            self.quote_identifier(table).to_string()
        };

        Ok(format!(
            "DELETE FROM {} WHERE {};",
            table_name, where_clause
        ))
    }

    fn format_value(&self, val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => format!("'{}'", self.escape_sql_string(s)),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                format!("'{}'", self.escape_sql_string(&val.to_string()))
            }
        }
    }

    fn format_where_condition(&self, val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Null => "IS NULL".to_string(),
            serde_json::Value::Bool(b) => if *b { "= true" } else { "= false" }.to_string(),
            serde_json::Value::Number(n) => format!("= {}", n),
            serde_json::Value::String(s) => format!("= '{}'", self.escape_sql_string(s)),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                format!("= '{}'", self.escape_sql_string(&val.to_string()))
            }
        }
    }

    fn escape_sql_string(&self, s: &str) -> String {
        s.replace("'", "''")
    }
}

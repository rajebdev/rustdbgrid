use crate::db::traits::{CRUDQueryBuilder, QueryBuilder};
use crate::models::save_request::EditedRow;
use crate::models::table_request::*;
use anyhow::Result;
use std::collections::HashMap;

/// PostgreSQL Query Builder
pub struct PostgreSQLQueryBuilder;

impl QueryBuilder for PostgreSQLQueryBuilder {
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
        let table = &request.table;

        // If table is wrapped with RustDBGridQuery(), extract and return the inner query
        let trimmed = table.trim();
        if trimmed.starts_with("RustDBGridQuery(") && trimmed.ends_with(')') {
            let query = &trimmed[16..trimmed.len() - 1]; // Extract between RustDBGridQuery( and )
            return format!("({}) AS __query", query);
        }

        // Normal table - quote it
        let quoted_table = self.quote_identifier(table);

        match (&request.database, &request.schema) {
            (Some(db), Some(schema)) => format!(
                "{}.{}.{}",
                self.quote_identifier(db),
                self.quote_identifier(schema),
                quoted_table
            ),
            (None, Some(schema)) => format!("{}.{}", self.quote_identifier(schema), quoted_table),
            (Some(db), None) => format!("{}.{}", self.quote_identifier(db), quoted_table),
            (None, None) => quoted_table,
        }
    }

    fn build_where_clause(&self, filters: &[Filter]) -> Result<String> {
        let conditions: Vec<String> = filters
            .iter()
            .map(|f| self.build_filter_condition(f))
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

impl PostgreSQLQueryBuilder {
    fn build_filter_condition(&self, filter: &Filter) -> Result<String> {
        let column = self.quote_identifier(&filter.column);

        let condition = match &filter.operator {
            FilterOperator::Equals => match &filter.value {
                FilterValue::Single(v) => format!("{} = {}", column, self.format_value(v)),
                _ => anyhow::bail!("Equals operator requires single value"),
            },
            FilterOperator::NotEquals => match &filter.value {
                FilterValue::Single(v) => format!("{} != {}", column, self.format_value(v)),
                _ => anyhow::bail!("NotEquals operator requires single value"),
            },
            FilterOperator::In => match &filter.value {
                FilterValue::Multiple(values) => {
                    let values_str = values
                        .iter()
                        .map(|v| self.format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} IN ({})", column, values_str)
                }
                FilterValue::Single(serde_json::Value::Array(values)) => {
                    // Handle case where array is parsed as Single (due to serde(untagged))
                    let values_str = values
                        .iter()
                        .map(|v| self.format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} IN ({})", column, values_str)
                }
                _ => anyhow::bail!("In operator requires multiple values"),
            },
            FilterOperator::NotIn => match &filter.value {
                FilterValue::Multiple(values) => {
                    let values_str = values
                        .iter()
                        .map(|v| self.format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} NOT IN ({})", column, values_str)
                }
                FilterValue::Single(serde_json::Value::Array(values)) => {
                    // Handle case where array is parsed as Single (due to serde(untagged))
                    let values_str = values
                        .iter()
                        .map(|v| self.format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} NOT IN ({})", column, values_str)
                }
                _ => anyhow::bail!("NotIn operator requires multiple values"),
            },
            FilterOperator::Like => match &filter.value {
                FilterValue::Single(v) => {
                    format!("{} LIKE {}", column, self.format_value(v))
                }
                _ => anyhow::bail!("Like operator requires single value"),
            },
            FilterOperator::NotLike => match &filter.value {
                FilterValue::Single(v) => {
                    format!("{} NOT LIKE {}", column, self.format_value(v))
                }
                _ => anyhow::bail!("NotLike operator requires single value"),
            },
            FilterOperator::GreaterThan => match &filter.value {
                FilterValue::Single(v) => format!("{} > {}", column, self.format_value(v)),
                _ => anyhow::bail!("GreaterThan operator requires single value"),
            },
            FilterOperator::GreaterThanOrEqual => match &filter.value {
                FilterValue::Single(v) => format!("{} >= {}", column, self.format_value(v)),
                _ => anyhow::bail!("GreaterThanOrEqual operator requires single value"),
            },
            FilterOperator::LessThan => match &filter.value {
                FilterValue::Single(v) => format!("{} < {}", column, self.format_value(v)),
                _ => anyhow::bail!("LessThan operator requires single value"),
            },
            FilterOperator::LessThanOrEqual => match &filter.value {
                FilterValue::Single(v) => format!("{} <= {}", column, self.format_value(v)),
                _ => anyhow::bail!("LessThanOrEqual operator requires single value"),
            },
            FilterOperator::Between => match &filter.value {
                FilterValue::Range { from, to } => {
                    format!(
                        "{} BETWEEN {} AND {}",
                        column,
                        self.format_value(from),
                        self.format_value(to)
                    )
                }
                _ => anyhow::bail!("Between operator requires range value"),
            },
            FilterOperator::IsNull => format!("{} IS NULL", column),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", column),
        };

        Ok(condition)
    }

    fn format_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            _ => format!("'{}'", value.to_string().replace("'", "''")),
        }
    }
}

impl CRUDQueryBuilder for PostgreSQLQueryBuilder {
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

use crate::db::traits::{CRUDQueryBuilder, QueryBuilder};
use crate::models::save_request::EditedRow;
use crate::models::table_request::*;
use anyhow::Result;
use std::collections::HashMap;

pub struct MongoDBQueryBuilder;

impl QueryBuilder for MongoDBQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let mut query_doc = serde_json::json!({
            "collection": request.table,
            "operation": "find",
            "query": {},
            "options": {
                "limit": request.limit,
                "skip": request.offset
            }
        });

        if let Some(ref db) = request.database {
            query_doc["db"] = serde_json::json!(db);
        }

        if !request.filters.is_empty() {
            let filter = self.build_mongo_filter(&request.filters)?;
            query_doc["query"] = filter;
        }

        if !request.order_by.is_empty() {
            let sort = self.build_mongo_sort(&request.order_by);
            query_doc["options"]["sort"] = sort;
        }

        serde_json::to_string(&query_doc)
            .map_err(|e| anyhow::anyhow!("Failed to serialize MongoDB query: {}", e))
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        identifier.to_string()
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        request.table.clone()
    }

    fn build_where_clause(&self, _filters: &[Filter]) -> Result<String> {
        Ok("{}".to_string())
    }

    fn build_order_by_clause(&self, _order_by: &[OrderBy]) -> String {
        "{}".to_string()
    }

    fn build_pagination_clause(&self, _limit: usize, _offset: usize) -> String {
        "".to_string()
    }
}

impl MongoDBQueryBuilder {
    fn build_mongo_filter(&self, filters: &[Filter]) -> Result<serde_json::Value> {
        let mut filter_doc = serde_json::Map::new();

        for filter in filters {
            let field_filter = match &filter.operator {
                FilterOperator::Equals => match &filter.value {
                    FilterValue::Single(v) => v.clone(),
                    _ => continue,
                },
                FilterOperator::NotEquals => match &filter.value {
                    FilterValue::Single(v) => serde_json::json!({ "$ne": v }),
                    _ => continue,
                },
                FilterOperator::In => match &filter.value {
                    FilterValue::Multiple(v) => serde_json::json!({ "$in": v }),
                    FilterValue::Single(serde_json::Value::Array(v)) => {
                        // Handle case where array is parsed as Single (due to serde(untagged))
                        serde_json::json!({ "$in": v })
                    }
                    _ => continue,
                },
                FilterOperator::NotIn => match &filter.value {
                    FilterValue::Multiple(v) => serde_json::json!({ "$nin": v }),
                    FilterValue::Single(serde_json::Value::Array(v)) => {
                        // Handle case where array is parsed as Single (due to serde(untagged))
                        serde_json::json!({ "$nin": v })
                    }
                    _ => continue,
                },
                FilterOperator::Like => match &filter.value {
                    FilterValue::Single(v) => {
                        if let Some(s) = v.as_str() {
                            let pattern = s.replace('%', ".*").replace('_', ".");
                            serde_json::json!({ "$regex": pattern, "$options": "i" })
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                FilterOperator::NotLike => match &filter.value {
                    FilterValue::Single(v) => {
                        if let Some(s) = v.as_str() {
                            let pattern = s.replace('%', ".*").replace('_', ".");
                            serde_json::json!({ "$not": { "$regex": pattern, "$options": "i" } })
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                },
                FilterOperator::GreaterThan => match &filter.value {
                    FilterValue::Single(v) => serde_json::json!({ "$gt": v }),
                    _ => continue,
                },
                FilterOperator::GreaterThanOrEqual => match &filter.value {
                    FilterValue::Single(v) => serde_json::json!({ "$gte": v }),
                    _ => continue,
                },
                FilterOperator::LessThan => match &filter.value {
                    FilterValue::Single(v) => serde_json::json!({ "$lt": v }),
                    _ => continue,
                },
                FilterOperator::LessThanOrEqual => match &filter.value {
                    FilterValue::Single(v) => serde_json::json!({ "$lte": v }),
                    _ => continue,
                },
                FilterOperator::Between => match &filter.value {
                    FilterValue::Range { from, to } => {
                        serde_json::json!({ "$gte": from, "$lte": to })
                    }
                    _ => continue,
                },
                FilterOperator::IsNull => serde_json::json!({ "$eq": null }),
                FilterOperator::IsNotNull => serde_json::json!({ "$ne": null }),
            };

            filter_doc.insert(filter.column.clone(), field_filter);
        }

        Ok(serde_json::Value::Object(filter_doc))
    }

    fn build_mongo_sort(&self, order_by: &[OrderBy]) -> serde_json::Value {
        let mut sort_doc = serde_json::Map::new();

        for order in order_by {
            let direction = match order.direction {
                SortDirection::Asc => 1,
                SortDirection::Desc => -1,
            };
            sort_doc.insert(order.column.clone(), serde_json::json!(direction));
        }

        serde_json::Value::Object(sort_doc)
    }
}

impl CRUDQueryBuilder for MongoDBQueryBuilder {
    fn build_insert_query(
        &self,
        table: &str,
        _schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        _table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String> {
        if row.is_empty() {
            anyhow::bail!("Cannot insert empty row");
        }

        let query_doc = serde_json::json!({
            "collection": table,
            "operation": "insertOne",
            "document": serde_json::Value::Object(
                row.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            )
        });

        serde_json::to_string(&query_doc)
            .map_err(|e| anyhow::anyhow!("Failed to serialize MongoDB insert query: {}", e))
    }

    fn build_update_query(
        &self,
        table: &str,
        _schema: Option<&str>,
        edited_row: &EditedRow,
        primary_keys: &[&String],
        _table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String> {
        let updated_data = &edited_row.updated_data;
        let original_data = &edited_row.original_data;

        if updated_data.is_empty() {
            anyhow::bail!("Cannot update with no columns");
        }

        // Build filter based on primary keys
        let filter = if !primary_keys.is_empty() {
            let mut filter_doc = serde_json::Map::new();
            for pk in primary_keys {
                let val = original_data.get(pk.as_str()).ok_or_else(|| {
                    anyhow::anyhow!("Primary key {} not found in original data", pk)
                })?;
                filter_doc.insert(pk.to_string(), val.clone());
            }
            serde_json::Value::Object(filter_doc)
        } else {
            serde_json::Value::Object(
                original_data
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        };

        let query_doc = serde_json::json!({
            "collection": table,
            "operation": "updateOne",
            "filter": filter,
            "update": {
                "$set": serde_json::Value::Object(
                    updated_data.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                )
            }
        });

        serde_json::to_string(&query_doc)
            .map_err(|e| anyhow::anyhow!("Failed to serialize MongoDB update query: {}", e))
    }

    fn build_delete_query(
        &self,
        table: &str,
        _schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        primary_keys: &[&String],
    ) -> Result<String> {
        // Build filter based on primary keys
        let filter = if !primary_keys.is_empty() {
            let mut filter_doc = serde_json::Map::new();
            for pk in primary_keys {
                let val = row
                    .get(pk.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Primary key {} not found in row", pk))?;
                filter_doc.insert(pk.to_string(), val.clone());
            }
            serde_json::Value::Object(filter_doc)
        } else {
            serde_json::Value::Object(row.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        };

        let query_doc = serde_json::json!({
            "collection": table,
            "operation": "deleteOne",
            "filter": filter
        });

        serde_json::to_string(&query_doc)
            .map_err(|e| anyhow::anyhow!("Failed to serialize MongoDB delete query: {}", e))
    }

    fn format_value(&self, val: &serde_json::Value) -> String {
        val.to_string()
    }

    fn format_where_condition(&self, val: &serde_json::Value) -> String {
        val.to_string()
    }

    fn escape_sql_string(&self, s: &str) -> String {
        s.to_string()
    }
}

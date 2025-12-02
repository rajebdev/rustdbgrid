use crate::models::schema::*;
use anyhow::Result;
use sqlx::{PgPool, Row};

/// PostgreSQL metadata operations
pub struct PostgresMetadataOps;

impl PostgresMetadataOps {
    /// Get all views in database/schema
    pub async fn get_views(pool: &PgPool, schema: Option<&str>) -> Result<Vec<View>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND schemaname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT schemaname as schema, viewname as name FROM pg_views 
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema') {} ORDER BY schemaname, viewname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let views = rows
            .iter()
            .map(|row| {
                let schema: String = row.try_get("schema").unwrap_or_default();
                let name: String = row.try_get("name").unwrap_or_default();
                View {
                    schema: Some(schema),
                    name,
                }
            })
            .collect();

        Ok(views)
    }

    /// Get all indexes in database/schema
    pub async fn get_indexes(pool: &PgPool, schema: Option<&str>) -> Result<Vec<DbIndex>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND schemaname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT schemaname, tablename, indexname, indexdef FROM pg_indexes
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema') {} ORDER BY schemaname, tablename, indexname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let indexes = rows
            .iter()
            .map(|row| {
                let table_name: String = row.try_get("tablename").unwrap_or_default();
                let name: String = row.try_get("indexname").unwrap_or_default();
                let indexdef: String = row.try_get("indexdef").unwrap_or_default();
                let is_unique = indexdef.to_lowercase().contains("unique");

                let index_type = if indexdef.to_lowercase().contains("btree") {
                    Some("BTREE".to_string())
                } else if indexdef.to_lowercase().contains("hash") {
                    Some("HASH".to_string())
                } else if indexdef.to_lowercase().contains("gist") {
                    Some("GIST".to_string())
                } else if indexdef.to_lowercase().contains("gin") {
                    Some("GIN".to_string())
                } else {
                    Some("BTREE".to_string())
                };

                DbIndex {
                    name,
                    table_name,
                    columns: vec![],
                    is_unique,
                    index_type,
                    ascending: Some(true),
                    nullable: None,
                    extra: None,
                }
            })
            .collect();

        Ok(indexes)
    }

    /// Get all procedures/functions in database/schema
    pub async fn get_procedures(pool: &PgPool, schema: Option<&str>) -> Result<Vec<Procedure>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND n.nspname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT n.nspname as schema, p.proname as name,
                    CASE WHEN p.prokind = 'f' THEN 'FUNCTION' WHEN p.prokind = 'p' THEN 'PROCEDURE' ELSE 'FUNCTION' END as type,
                    p.oid::text as oid
            FROM pg_proc p JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE n.nspname NOT IN ('pg_catalog', 'information_schema') {} ORDER BY n.nspname, p.proname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let procedures = rows
            .iter()
            .map(|row| {
                let schema: String = row.try_get("schema").unwrap_or_default();
                let name: String = row.try_get("name").unwrap_or_default();
                let proc_type: String = row.try_get("type").unwrap_or_default();
                let oid: String = row.try_get("oid").unwrap_or_default();
                Procedure {
                    name,
                    schema: Some(schema),
                    procedure_type: Some(proc_type),
                    oid: Some(oid),
                }
            })
            .collect();

        Ok(procedures)
    }

    /// Get all triggers in database/schema
    pub async fn get_triggers(pool: &PgPool, schema: Option<&str>) -> Result<Vec<Trigger>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND n.nspname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT t.tgname as trigger_name, c.relname as table_name,
                    CASE WHEN t.tgtype & 2 = 2 THEN 'BEFORE' WHEN t.tgtype & 64 = 64 THEN 'INSTEAD OF' ELSE 'AFTER' END as timing,
                    CASE WHEN t.tgtype & 4 = 4 THEN 'INSERT' WHEN t.tgtype & 8 = 8 THEN 'DELETE' WHEN t.tgtype & 16 = 16 THEN 'UPDATE' ELSE 'UNKNOWN' END as event
            FROM pg_trigger t JOIN pg_class c ON t.tgrelid = c.oid JOIN pg_namespace n ON c.relnamespace = n.oid
            WHERE NOT t.tgisinternal AND n.nspname NOT IN ('pg_catalog', 'information_schema') {} ORDER BY n.nspname, c.relname, t.tgname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let triggers = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("trigger_name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let timing: String = row.try_get("timing").unwrap_or_default();
                let event: String = row.try_get("event").unwrap_or_default();

                Trigger {
                    name,
                    table_name,
                    timing,
                    event,
                    trigger_type: Some("ROW".to_string()),
                    description: None,
                }
            })
            .collect();

        Ok(triggers)
    }
}

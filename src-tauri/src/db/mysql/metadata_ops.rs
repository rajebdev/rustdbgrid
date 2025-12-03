use crate::models::schema::*;
use anyhow::Result;
use sqlx::{MySqlPool, Row};

/// MySQL metadata operations
pub struct MySqlMetadataOps;

impl MySqlMetadataOps {
    /// Get all views in database
    pub async fn get_views(pool: &MySqlPool, database: &str) -> Result<Vec<View>> {
        let query = format!(
            "SELECT TABLE_NAME as name 
            FROM information_schema.VIEWS 
            WHERE TABLE_SCHEMA = '{}' 
            ORDER BY TABLE_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let views = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                View { name, schema: None }
            })
            .collect();

        Ok(views)
    }

    /// Get all indexes in database
    pub async fn get_indexes(pool: &MySqlPool, database: &str) -> Result<Vec<DbIndex>> {
        let query = format!(
            "SELECT DISTINCT
                INDEX_NAME as name,
                TABLE_NAME as table_name,
                NON_UNIQUE,
                INDEX_TYPE as index_type,
                COLLATION as collation,
                NULLABLE,
                INDEX_COMMENT as extra
            FROM information_schema.STATISTICS 
            WHERE TABLE_SCHEMA = '{}' 
            ORDER BY TABLE_NAME, INDEX_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let indexes = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let non_unique: i32 = row.try_get("NON_UNIQUE").unwrap_or(1);
                let index_type: Option<String> = row.try_get("index_type").ok();
                let collation: Option<String> = row.try_get("collation").ok();
                let nullable: Option<String> = row.try_get("NULLABLE").ok();
                let extra: Option<String> = row.try_get("extra").ok();

                DbIndex {
                    name,
                    table_name,
                    columns: vec![],
                    is_unique: non_unique == 0,
                    index_type,
                    ascending: collation.as_ref().map(|c| c == "A"),
                    nullable: nullable.as_ref().map(|n| n == "YES"),
                    extra,
                }
            })
            .collect();

        Ok(indexes)
    }

    /// Get all procedures/functions in database
    pub async fn get_procedures(pool: &MySqlPool, database: &str) -> Result<Vec<Procedure>> {
        let query = format!(
            "SELECT 
                ROUTINE_NAME as name,
                ROUTINE_TYPE as procedure_type
            FROM information_schema.ROUTINES 
            WHERE ROUTINE_SCHEMA = '{}' 
            ORDER BY ROUTINE_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let procedures = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let procedure_type: Option<String> = row.try_get("procedure_type").ok();
                Procedure {
                    name,
                    schema: None,
                    procedure_type,
                    oid: None,
                }
            })
            .collect();

        Ok(procedures)
    }

    /// Get all triggers in database
    pub async fn get_triggers(pool: &MySqlPool, database: &str) -> Result<Vec<Trigger>> {
        let query = format!(
            "SELECT 
                TRIGGER_NAME as name,
                EVENT_OBJECT_TABLE as table_name,
                EVENT_MANIPULATION as event,
                ACTION_TIMING as timing,
                ACTION_ORIENTATION as trigger_type,
                ACTION_STATEMENT as description
            FROM information_schema.TRIGGERS 
            WHERE TRIGGER_SCHEMA = '{}' 
            ORDER BY TRIGGER_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let triggers = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let event: String = row.try_get("event").unwrap_or_default();
                let timing: String = row.try_get("timing").unwrap_or_default();
                let trigger_type: Option<String> = row.try_get("trigger_type").ok();
                let description: Option<String> = row.try_get("description").ok();

                Trigger {
                    name,
                    table_name,
                    event,
                    timing,
                    trigger_type,
                    description,
                }
            })
            .collect();

        Ok(triggers)
    }

    /// Get all events in database
    pub async fn get_events(pool: &MySqlPool, database: &str) -> Result<Vec<Event>> {
        let query = format!(
            "SELECT 
                EVENT_NAME as name,
                STATUS as status,
                INTERVAL_VALUE as interval_value,
                INTERVAL_FIELD as interval_field
            FROM information_schema.EVENTS 
            WHERE EVENT_SCHEMA = '{}' 
            ORDER BY EVENT_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let events = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let status: Option<String> = row.try_get("status").ok();
                let interval_value: Option<String> =
                    row.try_get::<String, _>("interval_value").ok();
                let interval_field: Option<String> = row.try_get("interval_field").ok();
                Event {
                    name,
                    status,
                    interval_value,
                    interval_field,
                }
            })
            .collect();

        Ok(events)
    }
}

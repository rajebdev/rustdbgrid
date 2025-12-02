use crate::models::schema::*;
use anyhow::Result;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

/// MSSQL metadata operations
pub struct MSSQLMetadataOps;

impl MSSQLMetadataOps {
    /// Get all views in database
    pub async fn get_views(
        pool: &Pool<ConnectionManager>,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<View>> {
        let mut conn = pool.get().await?;

        let query = if let Some(schema_name) = schema {
            format!(
                "SELECT 
                    TABLE_NAME as name,
                    TABLE_SCHEMA as schema_name
                FROM [{database}].INFORMATION_SCHEMA.VIEWS
                WHERE TABLE_SCHEMA = '{schema_name}'
                ORDER BY TABLE_NAME"
            )
        } else {
            format!(
                "SELECT 
                    TABLE_NAME as name,
                    TABLE_SCHEMA as schema_name
                FROM [{database}].INFORMATION_SCHEMA.VIEWS
                ORDER BY TABLE_NAME"
            )
        };

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let views = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("name")?.to_string();
                let schema = row.get::<&str, _>("schema_name").map(|s| s.to_string());
                Some(View { name, schema })
            })
            .collect();

        Ok(views)
    }

    /// Get all indexes in database
    pub async fn get_indexes(
        pool: &Pool<ConnectionManager>,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<DbIndex>> {
        let mut conn = pool.get().await?;

        let query = if let Some(schema_name) = schema {
            format!(
                "SELECT 
                    i.name as index_name,
                    t.name as table_name,
                    s.name as schema_name,
                    i.type_desc as index_type,
                    i.is_unique
                FROM [{database}].sys.indexes i
                INNER JOIN [{database}].sys.tables t ON i.object_id = t.object_id
                INNER JOIN [{database}].sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = '{schema_name}' AND i.name IS NOT NULL
                ORDER BY s.name, t.name, i.name"
            )
        } else {
            format!(
                "SELECT 
                    i.name as index_name,
                    t.name as table_name,
                    s.name as schema_name,
                    i.type_desc as index_type,
                    i.is_unique
                FROM [{database}].sys.indexes i
                INNER JOIN [{database}].sys.tables t ON i.object_id = t.object_id
                INNER JOIN [{database}].sys.schemas s ON t.schema_id = s.schema_id
                WHERE i.name IS NOT NULL
                ORDER BY s.name, t.name, i.name"
            )
        };

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let indexes = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("index_name")?.to_string();
                let table_name = row.get::<&str, _>("table_name")?.to_string();
                let index_type = row.get::<&str, _>("index_type").map(|s| s.to_string());
                let is_unique: Option<bool> = row.get("is_unique");

                Some(DbIndex {
                    name,
                    table_name,
                    columns: vec![],
                    is_unique: is_unique.unwrap_or(false),
                    index_type,
                    ascending: None,
                    nullable: None,
                    extra: None,
                })
            })
            .collect();

        Ok(indexes)
    }

    /// Get all procedures in database
    pub async fn get_procedures(
        pool: &Pool<ConnectionManager>,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        let mut conn = pool.get().await?;

        let query = if let Some(schema_name) = schema {
            format!(
                "SELECT 
                    p.name as procedure_name,
                    s.name as schema_name,
                    'PROCEDURE' as routine_type
                FROM [{database}].sys.procedures p
                INNER JOIN [{database}].sys.schemas s ON p.schema_id = s.schema_id
                WHERE s.name = '{schema_name}' AND p.name NOT LIKE 'sp_%'
                ORDER BY s.name, p.name"
            )
        } else {
            format!(
                "SELECT 
                    p.name as procedure_name,
                    s.name as schema_name,
                    'PROCEDURE' as routine_type
                FROM [{database}].sys.procedures p
                INNER JOIN [{database}].sys.schemas s ON p.schema_id = s.schema_id
                WHERE p.name NOT LIKE 'sp_%'
                ORDER BY s.name, p.name"
            )
        };

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let procedures = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("procedure_name")?.to_string();
                let schema = row.get::<&str, _>("schema_name").map(|s| s.to_string());
                let routine_type = row.get::<&str, _>("routine_type").map(|s| s.to_string());

                Some(Procedure {
                    name,
                    schema,
                    procedure_type: routine_type,
                    oid: None,
                })
            })
            .collect();

        Ok(procedures)
    }

    /// Get all triggers in database
    pub async fn get_triggers(
        pool: &Pool<ConnectionManager>,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Trigger>> {
        let mut conn = pool.get().await?;

        let query = if let Some(schema_name) = schema {
            format!(
                "SELECT 
                    tr.name as trigger_name,
                    s.name as schema_name,
                    t.name as table_name,
                    'TRIGGER' as trigger_type
                FROM [{database}].sys.triggers tr
                INNER JOIN [{database}].sys.tables t ON tr.parent_id = t.object_id
                INNER JOIN [{database}].sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = '{schema_name}' AND tr.is_ms_shipped = 0
                ORDER BY s.name, t.name, tr.name"
            )
        } else {
            format!(
                "SELECT 
                    tr.name as trigger_name,
                    s.name as schema_name,
                    t.name as table_name,
                    'TRIGGER' as trigger_type
                FROM [{database}].sys.triggers tr
                INNER JOIN [{database}].sys.tables t ON tr.parent_id = t.object_id
                INNER JOIN [{database}].sys.schemas s ON t.schema_id = s.schema_id
                WHERE tr.is_ms_shipped = 0
                ORDER BY s.name, t.name, tr.name"
            )
        };

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let triggers = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("trigger_name")?.to_string();
                let table_name = row.get::<&str, _>("table_name").map(|s| s.to_string());

                Some(Trigger {
                    name,
                    table_name: table_name.unwrap_or_default(),
                    event: String::new(),
                    timing: String::new(),
                    trigger_type: None,
                    description: None,
                })
            })
            .collect();

        Ok(triggers)
    }
}

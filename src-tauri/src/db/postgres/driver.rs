use crate::db::traits::DatabaseConnection;
use crate::db::postgres::type_converter::{map_pg_type, extract_pg_value_typed, PgColType};
use crate::db::postgres::metadata_ops::*;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{Column as SqlxColumn, PgPool, Row, TypeInfo};
use std::collections::HashMap;
use std::time::Instant;

pub struct PostgresConnection {
    pool: Option<PgPool>,
}

impl PostgresConnection {
    pub fn new() -> Self {
        Self { pool: None }
    }
}

impl Default for PostgresConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username.as_ref().unwrap_or(&"postgres".to_string()),
            config.password.as_ref().unwrap_or(&"".to_string()),
            config.host,
            config.port,
            config.database.as_ref().unwrap_or(&"postgres".to_string())
        );

        self.pool = Some(PgPool::connect(&url).await?);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = &self.pool {
            pool.close().await;
            self.pool = None;
        }
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            sqlx::query("SELECT 1").fetch_one(pool).await?;
            Ok(true)
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let start = Instant::now();

        let rows = sqlx::query(query).fetch_all(pool).await?;
        let execution_time = start.elapsed().as_millis();

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: vec![],
                column_display_names: None,
                column_types: None,
                rows: vec![],
                rows_affected: None,
                execution_time,
                final_query: None,
            });
        }

        let mut column_name_counts: HashMap<String, usize> = HashMap::new();
        let mut display_names = Vec::new();
        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| {
                let base_name = SqlxColumn::name(c).to_string();
                display_names.push(base_name.clone());
                let count = column_name_counts.entry(base_name.clone()).or_insert(0);
                *count += 1;
                if *count == 1 {
                    base_name
                } else {
                    format!("{}_{}", base_name, count)
                }
            })
            .collect();

        let col_type_info: Vec<(String, bool, PgColType)> = rows[0]
            .columns()
            .iter()
            .map(|col| {
                let type_name = col.type_info().name();
                let (base_type, is_array) = if let Some(stripped) = type_name.strip_prefix('_') {
                    (stripped.to_lowercase(), true)
                } else if let Some(stripped) = type_name.strip_suffix("[]") {
                    (stripped.to_lowercase(), true)
                } else {
                    (type_name.to_lowercase(), false)
                };
                let col_type = map_pg_type(&base_type);
                (base_type, is_array, col_type)
            })
            .collect();

        let mut result_rows = Vec::with_capacity(rows.len());

        for row in rows {
            let mut row_map = HashMap::with_capacity(columns.len());
            for (i, col) in columns.iter().enumerate() {
                let (base_type, is_array, col_type) = &col_type_info[i];
                let value = extract_pg_value_typed(&row, i, *col_type, *is_array, base_type);
                row_map.insert(col.clone(), value);
            }
            result_rows.push(row_map);
        }

        Ok(QueryResult {
            columns,
            column_display_names: Some(display_names),
            column_types: None,
            rows: result_rows,
            rows_affected: None,
            execution_time,
            final_query: None,
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let rows = sqlx::query(
            "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname",
        )
        .fetch_all(pool)
        .await?;

        let databases = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get(0).unwrap_or_default();
                Database { name }
            })
            .collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, _database: &str) -> Result<Vec<Table>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let query = "SELECT schemaname, tablename, pg_total_relation_size(schemaname||'.'||tablename) as size_bytes
            FROM pg_tables 
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY schemaname, tablename";

        let rows = sqlx::query(query).fetch_all(pool).await?;

        let tables = rows
            .iter()
            .map(|row| {
                let schema: String = row.try_get("schemaname").unwrap_or_default();
                let name: String = row.try_get("tablename").unwrap_or_default();
                let size_bytes: Option<i64> = row.try_get("size_bytes").ok();

                Table {
                    name,
                    schema: Some(schema),
                    size_bytes: size_bytes.map(|v| if v >= 0 { v as u64 } else { 0 }),
                }
            })
            .collect();

        Ok(tables)
    }

    async fn get_table_schema(&mut self, _database: &str, table: &str) -> Result<TableSchema> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT c.column_name, c.data_type, c.character_maximum_length, c.numeric_precision,
                    c.numeric_scale, c.is_nullable, c.column_default, c.ordinal_position,
                    COALESCE(tc.constraint_type = 'PRIMARY KEY', false) as is_primary
            FROM information_schema.columns c
            LEFT JOIN information_schema.constraint_column_usage ccu 
                ON c.column_name = ccu.column_name AND c.table_schema = ccu.table_schema AND c.table_name = ccu.table_name
            LEFT JOIN information_schema.table_constraints tc 
                ON ccu.constraint_name = tc.constraint_name AND tc.constraint_type = 'PRIMARY KEY'
            WHERE c.table_schema = '{}' AND c.table_name = '{}'
            ORDER BY c.ordinal_position",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("column_name").unwrap_or_default();
                let mut data_type: String = row.try_get("data_type").unwrap_or_default();
                let is_nullable: String = row.try_get("is_nullable").unwrap_or_default();
                let default_value: Option<String> = row.try_get("column_default").ok();
                let is_primary_key: bool = row.try_get("is_primary").unwrap_or(false);

                let char_max_length: Option<i32> = row.try_get("character_maximum_length").ok();
                let numeric_precision: Option<i32> = row.try_get("numeric_precision").ok();
                let numeric_scale: Option<i32> = row.try_get("numeric_scale").ok();

                data_type = match data_type.as_str() {
                    "character varying" => {
                        if let Some(length) = char_max_length {
                            format!("varchar({})", length)
                        } else {
                            "varchar".to_string()
                        }
                    }
                    "character" => {
                        if let Some(length) = char_max_length {
                            format!("char({})", length)
                        } else {
                            "char".to_string()
                        }
                    }
                    "numeric" | "decimal" => match (numeric_precision, numeric_scale) {
                        (Some(p), Some(s)) => format!("numeric({},{})", p, s),
                        (Some(p), None) => format!("numeric({})", p),
                        _ => data_type,
                    },
                    "double precision" => "float8".to_string(),
                    "smallint" => "int2".to_string(),
                    "integer" => "int4".to_string(),
                    "bigint" => "int8".to_string(),
                    "boolean" => "bool".to_string(),
                    "timestamp without time zone" => "timestamp".to_string(),
                    "timestamp with time zone" => "timestamptz".to_string(),
                    "time without time zone" => "time".to_string(),
                    "time with time zone" => "timetz".to_string(),
                    _ => data_type,
                };

                let is_auto_increment = default_value
                    .as_ref()
                    .map(|v| v.contains("nextval"))
                    .unwrap_or(false);

                Column {
                    name,
                    data_type,
                    nullable: is_nullable == "YES",
                    default_value,
                    is_primary_key,
                    is_auto_increment,
                }
            })
            .collect();

        let index_query = format!(
            "SELECT indexname, indexdef FROM pg_indexes 
            WHERE schemaname = '{}' AND tablename = '{}'",
            schema_lower, table_lower
        );

        let index_rows = sqlx::query(&index_query).fetch_all(pool).await?;
        let indexes = index_rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("indexname").unwrap_or_default();
                let indexdef: String = row.try_get("indexdef").unwrap_or_default();
                let is_unique = indexdef.contains("UNIQUE");

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

                Index {
                    name,
                    columns: vec![],
                    is_unique,
                    index_type,
                    ascending: Some(true),
                    nullable: None,
                    extra: None,
                }
            })
            .collect();

        let fk_query = format!(
            "SELECT tc.constraint_name, kcu.column_name, ccu.table_name AS foreign_table_name,
                    ccu.column_name AS foreign_column_name, rc.update_rule, rc.delete_rule,
                    tc.table_schema as owner
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name AND ccu.table_schema = tc.table_schema
            LEFT JOIN information_schema.referential_constraints AS rc
                ON tc.constraint_name = rc.constraint_name AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = '{}' AND tc.table_name = '{}'",
            schema_lower, table_lower
        );

        let fk_rows = sqlx::query(&fk_query).fetch_all(pool).await?;
        let foreign_keys = fk_rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("constraint_name").unwrap_or_default();
                let column: String = row.try_get("column_name").unwrap_or_default();
                let referenced_table: String =
                    row.try_get("foreign_table_name").unwrap_or_default();
                let referenced_column: String =
                    row.try_get("foreign_column_name").unwrap_or_default();
                let on_update: Option<String> = row.try_get("update_rule").ok();
                let on_delete: Option<String> = row.try_get("delete_rule").ok();
                let owner: Option<String> = row.try_get("owner").ok();

                ForeignKey {
                    name,
                    column,
                    referenced_table,
                    referenced_column,
                    owner,
                    ref_object_type: Some("TABLE".to_string()),
                    on_delete,
                    on_update,
                }
            })
            .collect();

        Ok(TableSchema {
            table_name: table.to_string(),
            columns,
            indexes,
            foreign_keys,
        })
    }

    async fn get_table_relationships(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<TableRelationship>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT tc.constraint_name, tc.table_name, kcu.column_name,
                    ccu.table_name AS referenced_table_name, ccu.column_name AS referenced_column_name,
                    rc.update_rule, rc.delete_rule, 'FOREIGN_KEY' as relationship_type
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name AND ccu.table_schema = tc.table_schema
            LEFT JOIN information_schema.referential_constraints AS rc
                ON tc.constraint_name = rc.constraint_name AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = '{}' AND tc.table_name = '{}'
            UNION ALL
            SELECT tc.constraint_name, tc.table_name, kcu.column_name,
                    ccu.table_name AS referenced_table_name, ccu.column_name AS referenced_column_name,
                    rc.update_rule, rc.delete_rule, 'REFERENCED_BY' as relationship_type
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name AND ccu.table_schema = tc.table_schema
            LEFT JOIN information_schema.referential_constraints AS rc
                ON tc.constraint_name = rc.constraint_name AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = '{}' AND ccu.table_name = '{}'",
            schema_lower, table_lower, schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let relationships = rows
            .iter()
            .map(|row| {
                let constraint_name: String = row.try_get("constraint_name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let column_name: String = row.try_get("column_name").unwrap_or_default();
                let referenced_table_name: String =
                    row.try_get("referenced_table_name").unwrap_or_default();
                let referenced_column_name: String =
                    row.try_get("referenced_column_name").unwrap_or_default();
                let relationship_type: String =
                    row.try_get("relationship_type").unwrap_or_default();
                let on_update: Option<String> = row.try_get("update_rule").ok();
                let on_delete: Option<String> = row.try_get("delete_rule").ok();

                TableRelationship {
                    constraint_name,
                    table_name,
                    column_name,
                    referenced_table_name,
                    referenced_column_name,
                    relationship_type,
                    owner: Some(schema.clone()),
                    ref_object_type: Some("TABLE".to_string()),
                    on_delete,
                    on_update,
                }
            })
            .collect();

        Ok(relationships)
    }

    async fn get_views(&mut self, _database: &str, schema: Option<&str>) -> Result<Vec<View>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        PostgresMetadataOps::get_views(pool, schema).await
    }

    async fn get_indexes(&mut self, _database: &str, schema: Option<&str>) -> Result<Vec<DbIndex>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        PostgresMetadataOps::get_indexes(pool, schema).await
    }

    async fn get_procedures(
        &mut self,
        _database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        PostgresMetadataOps::get_procedures(pool, schema).await
    }

    async fn get_triggers(
        &mut self,
        _database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Trigger>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        PostgresMetadataOps::get_triggers(pool, schema).await
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// PostgreSQL-specific implementations
impl PostgresConnection {
    pub async fn get_pg_constraints(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgConstraint>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT
                con.conname as constraint_name,
                CASE con.contype
                    WHEN 'c' THEN 'CHECK'
                    WHEN 'f' THEN 'FOREIGN KEY'
                    WHEN 'p' THEN 'PRIMARY KEY'
                    WHEN 'u' THEN 'UNIQUE'
                    WHEN 't' THEN 'TRIGGER'
                    WHEN 'x' THEN 'EXCLUSION'
                    ELSE con.contype::text
                END as constraint_type,
                array_to_string(ARRAY(
                    SELECT a.attname
                    FROM unnest(con.conkey) AS u(attnum)
                    JOIN pg_attribute AS a ON a.attnum = u.attnum AND a.attrelid = con.conrelid
                ), ', ') as attributes,
                pg_get_constraintdef(con.oid) as expression,
                pg_catalog.obj_description(con.oid, 'pg_constraint') as comment,
                n.nspname as owner
            FROM pg_constraint con
            JOIN pg_class c ON c.oid = con.conrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND con.contype IN ('c', 'p', 'u', 'x')
            ORDER BY con.contype, con.conname",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let constraints: Vec<crate::models::schema::PgConstraint> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("constraint_name").unwrap_or_default();
                let constraint_type: String = row.try_get("constraint_type").unwrap_or_default();
                let attribute: String = row.try_get("attributes").unwrap_or_default();
                let expression: Option<String> = row.try_get("expression").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let owner: Option<String> = row.try_get("owner").ok();

                crate::models::schema::PgConstraint {
                    name,
                    attribute,
                    owner,
                    constraint_type,
                    expression,
                    comment,
                }
            })
            .collect();

        Ok(constraints)
    }

    pub async fn get_pg_foreign_keys(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgForeignKey>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT
                con.conname as fk_name,
                array_to_string(ARRAY(
                    SELECT a.attname
                    FROM unnest(con.conkey) AS u(attnum)
                    JOIN pg_attribute AS a ON a.attnum = u.attnum AND a.attrelid = con.conrelid
                ), ', ') as attributes,
                array_to_string(ARRAY(
                    SELECT a.attname
                    FROM unnest(con.confkey) AS u(attnum)
                    JOIN pg_attribute AS a ON a.attnum = u.attnum AND a.attrelid = con.confrelid
                ), ', ') as reference_columns,
                nf.nspname || '.' || cf.relname as associated_entity,
                CASE con.confmatchtype
                    WHEN 'f' THEN 'FULL'
                    WHEN 'p' THEN 'PARTIAL'
                    WHEN 's' THEN 'SIMPLE'
                    ELSE 'NONE'
                END as match_type,
                CASE con.confdeltype
                    WHEN 'a' THEN 'NO ACTION'
                    WHEN 'r' THEN 'RESTRICT'
                    WHEN 'c' THEN 'CASCADE'
                    WHEN 'n' THEN 'SET NULL'
                    WHEN 'd' THEN 'SET DEFAULT'
                    ELSE 'NO ACTION'
                END as delete_rule,
                CASE con.confupdtype
                    WHEN 'a' THEN 'NO ACTION'
                    WHEN 'r' THEN 'RESTRICT'
                    WHEN 'c' THEN 'CASCADE'
                    WHEN 'n' THEN 'SET NULL'
                    WHEN 'd' THEN 'SET DEFAULT'
                    ELSE 'NO ACTION'
                END as update_rule,
                pg_catalog.obj_description(con.oid, 'pg_constraint') as comment,
                n.nspname as owner,
                'FOREIGN KEY' as fk_type
            FROM pg_constraint con
            JOIN pg_class c ON c.oid = con.conrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_class cf ON cf.oid = con.confrelid
            JOIN pg_namespace nf ON nf.oid = cf.relnamespace
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND con.contype = 'f'
            ORDER BY con.conname",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let foreign_keys: Vec<crate::models::schema::PgForeignKey> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("fk_name").unwrap_or_default();
                let attribute: String = row.try_get("attributes").unwrap_or_default();
                let reference_column: String = row.try_get("reference_columns").unwrap_or_default();
                let associated_entity: String =
                    row.try_get("associated_entity").unwrap_or_default();
                let match_type: Option<String> = row.try_get("match_type").ok();
                let delete_rule: Option<String> = row.try_get("delete_rule").ok();
                let update_rule: Option<String> = row.try_get("update_rule").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let owner: Option<String> = row.try_get("owner").ok();
                let fk_type: String = row.try_get("fk_type").unwrap_or_default();

                crate::models::schema::PgForeignKey {
                    name,
                    attribute,
                    owner,
                    fk_type,
                    reference_column,
                    associated_entity,
                    match_type,
                    delete_rule,
                    update_rule,
                    comment,
                }
            })
            .collect();

        Ok(foreign_keys)
    }

    pub async fn get_pg_indexes(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgIndex>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT
                i.relname as idx_name,
                c.relname as table_name,
                a.attname as column_name,
                ix.indisunique as is_unique,
                am.amname as index_type,
                CASE
                    WHEN ix.indoption[array_position(ix.indkey::int[], a.attnum::int) - 1] & 1 = 1 THEN false
                    ELSE true
                END as ascending,
                NOT a.attnotnull as nullable,
                opc.opcname as operator_class,
                pg_get_expr(ix.indpred, ix.indrelid) as predicate
            FROM pg_index ix
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_class c ON c.oid = ix.indrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_am am ON am.oid = i.relam
            CROSS JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS u(attnum, ord)
            JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = u.attnum
            LEFT JOIN pg_opclass opc ON opc.oid = ix.indclass[u.ord - 1]
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND NOT ix.indisprimary
            ORDER BY i.relname, u.ord",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let indexes: Vec<crate::models::schema::PgIndex> = rows
            .iter()
            .map(|row| {
                let idx_name: String = row.try_get("idx_name").unwrap_or_default();
                let table: String = row.try_get("table_name").unwrap_or_default();
                let column: String = row.try_get("column_name").unwrap_or_default();
                let unique: bool = row.try_get("is_unique").unwrap_or(false);
                let ascending: Option<bool> = row.try_get("ascending").ok();
                let nullable: Option<bool> = row.try_get("nullable").ok();
                let operator_class: Option<String> = row.try_get("operator_class").ok();
                let predicate: Option<String> = row.try_get("predicate").ok();

                crate::models::schema::PgIndex {
                    column,
                    idx_name,
                    table,
                    ascending,
                    nullable,
                    unique,
                    operator_class,
                    predicate,
                }
            })
            .collect();

        Ok(indexes)
    }

    pub async fn get_pg_references(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgReference>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT
                con.conname as ref_name,
                'FOREIGN KEY' as ref_type,
                n.nspname || '.' || c.relname as associated_entity,
                NULL::int as sequence_num,
                pg_catalog.obj_description(con.oid, 'pg_constraint') as comment,
                n.nspname as owner
            FROM pg_constraint con
            JOIN pg_class c ON c.oid = con.conrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_class cf ON cf.oid = con.confrelid
            JOIN pg_namespace nf ON nf.oid = cf.relnamespace
            WHERE nf.nspname = '{}'
                AND cf.relname = '{}'
                AND con.contype = 'f'
            ORDER BY con.conname",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let references: Vec<crate::models::schema::PgReference> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("ref_name").unwrap_or_default();
                let ref_type: String = row.try_get("ref_type").unwrap_or_default();
                let associated_entity: String =
                    row.try_get("associated_entity").unwrap_or_default();
                let sequence_num: Option<i32> = row.try_get("sequence_num").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let owner: Option<String> = row.try_get("owner").ok();

                crate::models::schema::PgReference {
                    name,
                    owner,
                    ref_type,
                    comment,
                    associated_entity,
                    sequence_num,
                }
            })
            .collect();

        Ok(references)
    }

    pub async fn get_pg_partitions(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgPartition>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                c.relname as table_name,
                c.oid::text as object_id,
                n.nspname as owner,
                spcname as tablespace,
                c.reltuples::int8 as rowcount_estimate,
                c.relrowsecurity as has_row_level_security,
                (SELECT count(*) FROM pg_inherits WHERE inhparent = c.oid) as partition_count,
                CASE 
                    WHEN c.relkind = 'p' THEN 
                        pg_get_partkeydef(c.oid)
                    ELSE NULL
                END as partition_by,
                pg_catalog.obj_description(c.oid, 'pg_class') as comment,
                c.reloptions as extra_options
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            LEFT JOIN pg_tablespace ts ON ts.oid = c.reltablespace
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND c.relkind IN ('r', 'p')",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let partitions: Vec<crate::models::schema::PgPartition> = rows
            .iter()
            .map(|row| {
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let object_id: Option<String> = row.try_get("object_id").ok();
                let owner: Option<String> = row.try_get("owner").ok();
                let tablespace: Option<String> = row.try_get("tablespace").ok();
                let rowcount_estimate: Option<i64> = row.try_get("rowcount_estimate").ok();
                let has_row_level_security: bool =
                    row.try_get("has_row_level_security").unwrap_or(false);
                let partitions: Option<i32> = row.try_get("partition_count").ok();
                let partition_by: Option<String> = row.try_get("partition_by").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let extra_options: Option<String> = row
                    .try_get::<Option<Vec<String>>, _>("extra_options")
                    .ok()
                    .flatten()
                    .map(|opts| opts.join(", "));

                crate::models::schema::PgPartition {
                    table_name,
                    object_id,
                    owner,
                    tablespace,
                    rowcount_estimate,
                    has_row_level_security,
                    partitions,
                    partition_by: partition_by.clone(),
                    partitions_expression: partition_by,
                    extra_options,
                    comment,
                }
            })
            .collect();

        Ok(partitions)
    }
}

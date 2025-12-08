use crate::models::{connection::*, query_result::*, save_request::*, schema::*, table_request::*};
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;

/// Core trait for database connections
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn test_connection(&mut self) -> Result<bool>;
    async fn execute_query(&mut self, query: &str) -> Result<QueryResult>;
    async fn execute_update(&mut self, query: &str) -> Result<u64>;
    async fn get_databases(&mut self) -> Result<Vec<Database>>;
    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>>;
    async fn get_table_schema(&mut self, database: &str, table: &str) -> Result<TableSchema>;

    async fn get_table_relationships(
        &mut self,
        _database: &str,
        _table: &str,
    ) -> Result<Vec<TableRelationship>> {
        Ok(vec![])
    }

    async fn get_table_statistics(
        &mut self,
        _database: &str,
        _table: &str,
    ) -> Result<TableStatistics> {
        Ok(TableStatistics {
            row_count: None,
            avg_row_length: None,
            data_length: None,
            max_data_length: None,
            data_free: None,
            index_length: None,
            row_format: None,
            create_time: None,
            update_time: None,
            check_time: None,
            collation: None,
            checksum: None,
            engine: None,
            comment: None,
            table_size: None,
            pages: None,
        })
    }

    async fn get_views(&mut self, _database: &str, _schema: Option<&str>) -> Result<Vec<View>> {
        Ok(vec![])
    }

    async fn get_indexes(
        &mut self,
        _database: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<DbIndex>> {
        Ok(vec![])
    }

    async fn get_procedures(
        &mut self,
        _database: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        Ok(vec![])
    }

    async fn get_triggers(
        &mut self,
        _database: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<Trigger>> {
        Ok(vec![])
    }

    async fn get_events(&mut self, _database: &str, _schema: Option<&str>) -> Result<Vec<Event>> {
        Ok(vec![])
    }

    async fn get_procedure_source(
        &mut self,
        _database: &str,
        _procedure_name: &str,
        _procedure_type: Option<String>,
        _schema: Option<String>,
    ) -> Result<String> {
        Ok("-- Source code not available".to_string())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Trait for building SQL queries from JSON request structure
pub trait QueryBuilder {
    /// Build a SELECT query with filters, sorting, and pagination
    fn build_select_query(&self, request: &QueryRequest) -> Result<String>;

    /// Quote identifier (table, column, schema names) appropriately for the database
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Format the table name with database and schema if provided
    fn format_table_name(&self, request: &QueryRequest) -> String;

    /// Build WHERE clause from filters
    fn build_where_clause(&self, filters: &[Filter]) -> Result<String>;

    /// Build ORDER BY clause
    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String;

    /// Build LIMIT and OFFSET clause
    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String;
}

/// Factory function to create a database connection based on type
pub fn create_connection(db_type: &DatabaseType) -> Box<dyn DatabaseConnection> {
    match db_type {
        DatabaseType::MySQL => Box::new(crate::db::mysql::MySQLConnection::new()),
        DatabaseType::PostgreSQL => Box::new(crate::db::postgres::PostgresConnection::new()),
        DatabaseType::MongoDB => Box::new(crate::db::mongodb::MongoDBConnection::new()),
        DatabaseType::Redis => Box::new(crate::db::redis::RedisConnection::new()),
        DatabaseType::Ignite => Box::new(crate::db::ignite::IgniteConnection::new()),
        DatabaseType::MSSQL => Box::new(crate::db::mssql::MSSQLConnection::new()),
    }
}

/// Trait for building CRUD (INSERT, UPDATE, DELETE) queries
pub trait CRUDQueryBuilder: QueryBuilder + Send + Sync {
    /// Build an INSERT query with column ordering from table schema
    fn build_insert_query(
        &self,
        table: &str,
        schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String>;

    /// Build an UPDATE query with column ordering from table schema
    fn build_update_query(
        &self,
        table: &str,
        schema: Option<&str>,
        edited_row: &EditedRow,
        primary_keys: &[&String],
        table_schema: &crate::models::schema::TableSchema,
    ) -> Result<String>;

    /// Build a DELETE query
    fn build_delete_query(
        &self,
        table: &str,
        schema: Option<&str>,
        row: &HashMap<String, serde_json::Value>,
        primary_keys: &[&String],
    ) -> Result<String>;

    /// Format a value for SQL based on its JSON type
    fn format_value(&self, val: &serde_json::Value) -> String;

    /// Format a WHERE condition based on JSON value
    fn format_where_condition(&self, val: &serde_json::Value) -> String;

    /// Escape SQL string literals
    fn escape_sql_string(&self, s: &str) -> String;
}

/// Get appropriate query builder for database type
pub fn get_query_builder(db_type: &DatabaseType) -> Box<dyn QueryBuilder> {
    match db_type {
        DatabaseType::MySQL => Box::new(crate::db::mysql::MySQLQueryBuilder),
        DatabaseType::PostgreSQL => Box::new(crate::db::postgres::PostgreSQLQueryBuilder),
        DatabaseType::MSSQL => Box::new(crate::db::mssql::MSSQLQueryBuilder),
        DatabaseType::MongoDB => Box::new(crate::db::mongodb::MongoDBQueryBuilder),
        DatabaseType::Redis => Box::new(crate::db::redis::RedisQueryBuilder),
        DatabaseType::Ignite => Box::new(crate::db::ignite::IgniteQueryBuilder),
    }
}

/// Get appropriate CRUD query builder for database type
pub fn get_crud_query_builder(db_type: &DatabaseType) -> Box<dyn CRUDQueryBuilder> {
    match db_type {
        DatabaseType::MySQL => Box::new(crate::db::mysql::MySQLQueryBuilder),
        DatabaseType::PostgreSQL => Box::new(crate::db::postgres::PostgreSQLQueryBuilder),
        DatabaseType::MSSQL => Box::new(crate::db::mssql::MSSQLQueryBuilder),
        DatabaseType::MongoDB => Box::new(crate::db::mongodb::MongoDBQueryBuilder),
        DatabaseType::Redis => Box::new(crate::db::redis::RedisQueryBuilder),
        DatabaseType::Ignite => Box::new(crate::db::ignite::IgniteQueryBuilder),
    }
}

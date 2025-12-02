use crate::models::{connection::*, query_result::*, schema::*, table_request::*};
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;

/// Core trait for database connections
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn test_connection(&mut self) -> Result<bool>;
    async fn execute_query(&mut self, query: &str) -> Result<QueryResult>;
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

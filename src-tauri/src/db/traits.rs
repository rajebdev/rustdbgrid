use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::Result;
use async_trait::async_trait;

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
        // Default implementation returns empty statistics
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
        })
    }
    async fn get_table_data(
        &mut self,
        database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult>;

    // MySQL-specific methods with default implementations returning empty
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
}

pub fn create_connection(db_type: &DatabaseType) -> Box<dyn DatabaseConnection> {
    match db_type {
        DatabaseType::MySQL => Box::new(crate::db::mysql::MySQLConnection::new()),
        DatabaseType::PostgreSQL => Box::new(crate::db::postgres::PostgresConnection::new()),
        DatabaseType::MongoDB => Box::new(crate::db::mongodb::MongoDBConnection::new()),
        DatabaseType::Redis => Box::new(crate::db::redis::RedisConnection::new()),
        // Use Node.js bridge for Ignite (better compatibility)
        DatabaseType::Ignite => Box::new(crate::db::ignite_node::IgniteConnection::new()),
        DatabaseType::MSSQL => Box::new(crate::db::mssql::MSSQLConnection::new()),
    }
}

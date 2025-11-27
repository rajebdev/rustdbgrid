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
    async fn get_table_data(
        &mut self,
        database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult>;
}

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

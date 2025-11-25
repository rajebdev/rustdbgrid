use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{Result, anyhow};
use async_trait::async_trait;

pub struct IgniteConnection {
    // TODO: Implement Apache Ignite connection
}

impl IgniteConnection {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IgniteConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for IgniteConnection {
    async fn connect(&mut self, _config: &ConnectionConfig) -> Result<()> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }

    async fn execute_query(&mut self, _query: &str) -> Result<QueryResult> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }

    async fn get_tables(&mut self, _database: &str) -> Result<Vec<Table>> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }

    async fn get_table_schema(&mut self, _database: &str, _table: &str) -> Result<TableSchema> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }

    async fn get_table_data(&mut self, _database: &str, _table: &str, _limit: u32, _offset: u32) -> Result<QueryResult> {
        Err(anyhow!("Apache Ignite not yet implemented"))
    }
}

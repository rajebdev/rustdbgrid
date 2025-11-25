use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{Result, anyhow};
use async_trait::async_trait;

pub struct MongoDBConnection {
    // TODO: Implement MongoDB connection
}

impl MongoDBConnection {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MongoDBConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for MongoDBConnection {
    async fn connect(&mut self, _config: &ConnectionConfig) -> Result<()> {
        Err(anyhow!("MongoDB not yet implemented"))
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        Err(anyhow!("MongoDB not yet implemented"))
    }

    async fn execute_query(&mut self, _query: &str) -> Result<QueryResult> {
        Err(anyhow!("MongoDB not yet implemented"))
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        Err(anyhow!("MongoDB not yet implemented"))
    }

    async fn get_tables(&mut self, _database: &str) -> Result<Vec<Table>> {
        Err(anyhow!("MongoDB not yet implemented"))
    }

    async fn get_table_schema(&mut self, _database: &str, _table: &str) -> Result<TableSchema> {
        Err(anyhow!("MongoDB not yet implemented"))
    }

    async fn get_table_data(&mut self, _database: &str, _table: &str, _limit: u32, _offset: u32) -> Result<QueryResult> {
        Err(anyhow!("MongoDB not yet implemented"))
    }
}

use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client,
};
use std::collections::HashMap;
use std::time::Instant;

pub struct MongoDBConnection {
    client: Option<Client>,
    current_database: Option<String>,
}

impl MongoDBConnection {
    pub fn new() -> Self {
        Self {
            client: None,
            current_database: None,
        }
    }
}

impl Default for MongoDBConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for MongoDBConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let connection_string =
            if let (Some(username), Some(password)) = (&config.username, &config.password) {
                // URL-encode credentials to handle special characters
                let encoded_username = urlencoding::encode(username);
                let encoded_password = urlencoding::encode(password);

                // Specify authSource and authMechanism for proper authentication
                let auth_source = config.database.as_deref().unwrap_or("admin");
                format!(
                    "mongodb://{}:{}@{}:{}/?authSource={}&authMechanism=SCRAM-SHA-256",
                    encoded_username, encoded_password, config.host, config.port, auth_source
                )
            } else {
                format!("mongodb://{}:{}", config.host, config.port)
            };

        let client_options = ClientOptions::parse(&connection_string).await?;
        self.client = Some(Client::with_options(client_options)?);
        self.current_database = config.database.clone();
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.client = None;
        self.current_database = None;
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        if let Some(client) = &self.client {
            client.list_database_names().await?;
            Ok(true)
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let start = Instant::now();

        // Parse MongoDB query (expecting JSON format)
        // Format: { "db": "database_name", "collection": "collection_name", "operation": "find", "query": {...}, "options": {...} }
        let query_doc: serde_json::Value =
            serde_json::from_str(query).map_err(|e| anyhow!("Invalid JSON query: {}", e))?;

        let db_name = query_doc["db"]
            .as_str()
            .or(self.current_database.as_deref())
            .ok_or_else(|| anyhow!("Database name not specified"))?;
        let collection_name = query_doc["collection"]
            .as_str()
            .ok_or_else(|| anyhow!("Collection name not specified"))?;
        let operation = query_doc["operation"].as_str().unwrap_or("find");

        let db = client.database(db_name);
        let collection = db.collection::<Document>(collection_name);

        let mut rows = Vec::new();
        let mut columns = Vec::new();

        match operation {
            "find" => {
                let filter: Document = if let Some(query_filter) = query_doc.get("query") {
                    serde_json::from_value(query_filter.clone())
                        .map_err(|e| anyhow!("Invalid filter: {}", e))?
                } else {
                    doc! {}
                };

                let mut cursor = collection.find(filter).await?;

                use futures::stream::TryStreamExt;
                use mongodb::bson::Bson;

                while let Some(result) = cursor.try_next().await? {
                    let mut row_map = HashMap::new();

                    for (key, value) in result.iter() {
                        if columns.iter().all(|c| c != key) {
                            columns.push(key.clone());
                        }

                        let json_value = match value {
                            Bson::Double(v) => serde_json::json!(v),
                            Bson::String(v) => serde_json::json!(v),
                            Bson::Array(arr) => {
                                serde_json::to_value(arr).unwrap_or(serde_json::Value::Null)
                            }
                            Bson::Document(doc) => {
                                serde_json::to_value(doc).unwrap_or(serde_json::Value::Null)
                            }
                            Bson::Boolean(v) => serde_json::json!(v),
                            Bson::Null => serde_json::Value::Null,
                            Bson::Int32(v) => serde_json::json!(v),
                            Bson::Int64(v) => serde_json::json!(v),
                            Bson::ObjectId(oid) => serde_json::json!(oid.to_hex()),
                            Bson::DateTime(dt) => serde_json::json!(dt.to_string()),
                            _ => serde_json::json!(value.to_string()),
                        };

                        row_map.insert(key.clone(), json_value);
                    }
                    rows.push(row_map);
                }
            }
            "insertOne" | "insertMany" => {
                let docs = query_doc["documents"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Documents not specified for insert"))?;

                let bson_docs: Vec<Document> = docs
                    .iter()
                    .map(|d| {
                        serde_json::from_value(d.clone())
                            .map_err(|e| anyhow!("Invalid document: {}", e))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                if operation == "insertOne" && !bson_docs.is_empty() {
                    collection.insert_one(bson_docs[0].clone()).await?;
                } else {
                    collection.insert_many(bson_docs).await?;
                }
            }
            "updateOne" | "updateMany" => {
                let filter: Document = serde_json::from_value(query_doc["query"].clone())
                    .map_err(|e| anyhow!("Invalid filter: {}", e))?;
                let update: Document = serde_json::from_value(query_doc["update"].clone())
                    .map_err(|e| anyhow!("Invalid update: {}", e))?;

                if operation == "updateOne" {
                    collection.update_one(filter, update).await?;
                } else {
                    collection.update_many(filter, update).await?;
                }
            }
            "deleteOne" | "deleteMany" => {
                let filter: Document = serde_json::from_value(query_doc["query"].clone())
                    .map_err(|e| anyhow!("Invalid filter: {}", e))?;

                if operation == "deleteOne" {
                    collection.delete_one(filter).await?;
                } else {
                    collection.delete_many(filter).await?;
                }
            }
            _ => return Err(anyhow!("Unsupported operation: {}", operation)),
        }

        let execution_time = start.elapsed().as_millis();

        Ok(QueryResult {
            columns,
            column_types: None,
            rows,
            rows_affected: None,
            execution_time,
            final_query: Some(query.to_string()),
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let db_names = client.list_database_names().await?;

        let databases = db_names.into_iter().map(|name| Database { name }).collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let db = client.database(database);

        let collection_names = db.list_collection_names().await?;

        let mut tables = Vec::new();
        for name in collection_names {
            // Get collection stats for size
            let stats_result = db.run_command(doc! { "collStats": &name }).await;
            let size_bytes = if let Ok(stats) = stats_result {
                stats.get_i64("size").ok().map(|v| v as u64)
            } else {
                None
            };

            tables.push(Table {
                name,
                schema: None,
                size_bytes,
            });
        }

        Ok(tables)
    }

    async fn get_table_schema(&mut self, database: &str, table: &str) -> Result<TableSchema> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let db = client.database(database);
        let collection = db.collection::<Document>(table);

        // Sample a few documents to infer schema
        let mut cursor = collection.find(doc! {}).limit(100).await?;
        let mut field_types: HashMap<String, Vec<String>> = HashMap::new();

        use futures::stream::TryStreamExt;
        while let Some(doc) = cursor.try_next().await? {
            for (key, value) in doc.iter() {
                let type_name = match value {
                    mongodb::bson::Bson::Double(_) => "Double",
                    mongodb::bson::Bson::String(_) => "String",
                    mongodb::bson::Bson::Array(_) => "Array",
                    mongodb::bson::Bson::Document(_) => "Document",
                    mongodb::bson::Bson::Boolean(_) => "Boolean",
                    mongodb::bson::Bson::Null => "Null",
                    mongodb::bson::Bson::Int32(_) => "Int32",
                    mongodb::bson::Bson::Int64(_) => "Int64",
                    mongodb::bson::Bson::ObjectId(_) => "ObjectId",
                    mongodb::bson::Bson::DateTime(_) => "DateTime",
                    _ => "Mixed",
                };

                field_types
                    .entry(key.clone())
                    .or_default()
                    .push(type_name.to_string());
            }
        }

        let columns = field_types
            .into_iter()
            .map(|(name, types)| {
                let unique_types: Vec<String> = types
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                Column {
                    name: name.clone(),
                    data_type: unique_types.join(" | "),
                    nullable: true,
                    default_value: None,
                    is_primary_key: name == "_id",
                    is_auto_increment: false,
                }
            })
            .collect();

        Ok(TableSchema {
            table_name: table.to_string(),
            columns,
            indexes: vec![],
            foreign_keys: vec![],
        })
    }

    async fn get_table_data(
        &mut self,
        database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let db = client.database(database);
        let collection = db.collection::<Document>(table);

        let start = Instant::now();
        let mut cursor = collection
            .find(doc! {})
            .skip(offset as u64)
            .limit(limit as i64)
            .await?;

        let mut rows = Vec::new();
        let mut columns = Vec::new();

        use futures::stream::TryStreamExt;
        while let Some(doc) = cursor.try_next().await? {
            let mut row_map = HashMap::new();

            for (key, value) in doc.iter() {
                if columns.iter().all(|c| c != key) {
                    columns.push(key.clone());
                }

                let json_value = match value {
                    mongodb::bson::Bson::Double(v) => serde_json::json!(v),
                    mongodb::bson::Bson::String(v) => serde_json::json!(v),
                    mongodb::bson::Bson::Array(arr) => {
                        serde_json::to_value(arr).unwrap_or(serde_json::Value::Null)
                    }
                    mongodb::bson::Bson::Document(doc) => {
                        serde_json::to_value(doc).unwrap_or(serde_json::Value::Null)
                    }
                    mongodb::bson::Bson::Boolean(v) => serde_json::json!(v),
                    mongodb::bson::Bson::Null => serde_json::Value::Null,
                    mongodb::bson::Bson::Int32(v) => serde_json::json!(v),
                    mongodb::bson::Bson::Int64(v) => serde_json::json!(v),
                    mongodb::bson::Bson::ObjectId(oid) => serde_json::json!(oid.to_hex()),
                    mongodb::bson::Bson::DateTime(dt) => serde_json::json!(dt.to_string()),
                    _ => serde_json::json!(value.to_string()),
                };

                row_map.insert(key.clone(), json_value);
            }
            rows.push(row_map);
        }

        let execution_time = start.elapsed().as_millis();

        Ok(QueryResult {
            columns,
            column_types: None,
            rows,
            rows_affected: None,
            execution_time,
            final_query: None,
        })
    }
}

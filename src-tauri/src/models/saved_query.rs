use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub id: String,
    pub title: String,
    pub content: String,
    pub description: String,
    pub connection_id: Option<String>,
    pub database_name: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveQuery {
    pub tab_id: String,
    pub query: String,
    pub connection_id: Option<String>,
    pub database_name: Option<String>,
    pub saved_at: u64,
}

impl SavedQuery {
    pub fn new(
        title: String,
        content: String,
        description: String,
        connection_id: Option<String>,
        database_name: Option<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        SavedQuery {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            content,
            description,
            connection_id,
            database_name,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }
}

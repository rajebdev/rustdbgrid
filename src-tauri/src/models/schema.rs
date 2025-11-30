use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub schema: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub name: String,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbIndex {
    pub name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: Option<String>,
    pub ascending: Option<bool>,
    pub nullable: Option<bool>,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub name: String,
    pub schema: Option<String>,
    pub procedure_type: Option<String>,
    pub oid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub table_name: String,
    pub event: String,
    pub timing: String,
    pub trigger_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub status: Option<String>,
    pub interval_value: Option<String>,
    pub interval_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_auto_increment: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: Option<String>,
    pub ascending: Option<bool>,
    pub nullable: Option<bool>,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub name: String,
    pub column: String,
    pub referenced_table: String,
    pub referenced_column: String,
    pub owner: Option<String>,
    pub ref_object_type: Option<String>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRelationship {
    pub constraint_name: String,
    pub table_name: String,
    pub column_name: String,
    pub referenced_table_name: String,
    pub referenced_column_name: String,
    pub relationship_type: String, // "FOREIGN_KEY", "REFERENCED_BY", etc.
    pub owner: Option<String>,
    pub ref_object_type: Option<String>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStatistics {
    pub row_count: Option<i64>,
    pub avg_row_length: Option<i64>,
    pub data_length: Option<i64>,
    pub max_data_length: Option<i64>,
    pub data_free: Option<i64>,
    pub index_length: Option<i64>,
    pub row_format: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    pub check_time: Option<String>,
    pub collation: Option<String>,
    pub checksum: Option<String>,
    pub engine: Option<String>,
    pub comment: Option<String>,
    pub table_size: Option<String>,
    pub pages: Option<i64>,
}

// PostgreSQL specific models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgConstraint {
    pub name: String,
    pub attribute: String,
    pub owner: Option<String>,
    pub constraint_type: String, // CHECK, UNIQUE, PRIMARY KEY, etc.
    pub expression: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgForeignKey {
    pub name: String,
    pub attribute: String,
    pub owner: Option<String>,
    pub fk_type: String,
    pub reference_column: String,
    pub associated_entity: String,
    pub match_type: Option<String>,
    pub delete_rule: Option<String>,
    pub update_rule: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgIndex {
    pub column: String,
    pub idx_name: String,
    pub table: String,
    pub ascending: Option<bool>,
    pub nullable: Option<bool>,
    pub unique: bool,
    pub operator_class: Option<String>,
    pub predicate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgReference {
    pub name: String,
    pub owner: Option<String>,
    pub ref_type: String,
    pub comment: Option<String>,
    pub associated_entity: String,
    pub sequence_num: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgPartition {
    pub table_name: String,
    pub object_id: Option<String>,
    pub owner: Option<String>,
    pub tablespace: Option<String>,
    pub rowcount_estimate: Option<i64>,
    pub has_row_level_security: bool,
    pub partitions: Option<i32>,
    pub partition_by: Option<String>,
    pub partitions_expression: Option<String>,
    pub extra_options: Option<String>,
    pub comment: Option<String>,
}

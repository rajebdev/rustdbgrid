use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    MongoDB,
    Redis,
    Ignite,
    MSSQL,
}

impl Serialize for DatabaseType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            DatabaseType::MySQL => "MySQL",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::MongoDB => "MongoDB",
            DatabaseType::Redis => "Redis",
            DatabaseType::Ignite => "Ignite",
            DatabaseType::MSSQL => "MSSQL",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for DatabaseType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "MySQL" | "mysql" => DatabaseType::MySQL,
            "PostgreSQL" | "postgres" | "postgresql" => DatabaseType::PostgreSQL,
            "MongoDB" | "mongodb" => DatabaseType::MongoDB,
            "Redis" | "redis" => DatabaseType::Redis,
            "Ignite" | "ignite" => DatabaseType::Ignite,
            "MSSQL" | "mssql" | "sqlserver" => DatabaseType::MSSQL,
            unknown => {
                return Err(serde::de::Error::unknown_variant(
                    unknown,
                    &["MySQL", "PostgreSQL", "MongoDB", "Redis", "Ignite", "MSSQL"],
                ))
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub ssl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub success: bool,
    pub message: String,
}

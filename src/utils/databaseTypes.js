/**
 * Database Type Enum
 * Centralized constants for all supported database types
 */
export const DatabaseType = {
  POSTGRESQL: "PostgreSQL",
  MSSQL: "MSSQL",
  MYSQL: "MySQL",
  MONGODB: "MongoDB",
  REDIS: "Redis",
  IGNITE: "Ignite",
  MARIADB: "MariaDB",
  SQLITE: "SQLite",
  ORACLE: "Oracle",
  CASSANDRA: "Cassandra",
};

/**
 * Check if database type uses schemas
 */
export function usesSchemas(dbType) {
  return [DatabaseType.POSTGRESQL, DatabaseType.MSSQL].includes(dbType);
}

/**
 * Check if database type is NoSQL
 */
export function isNoSQL(dbType) {
  return [
    DatabaseType.MONGODB,
    DatabaseType.REDIS,
    DatabaseType.CASSANDRA,
  ].includes(dbType);
}

/**
 * Check if database type is relational
 */
export function isRelational(dbType) {
  return [
    DatabaseType.POSTGRESQL,
    DatabaseType.MSSQL,
    DatabaseType.MYSQL,
    DatabaseType.MARIADB,
    DatabaseType.SQLITE,
    DatabaseType.ORACLE,
  ].includes(dbType);
}

/**
 * Get display name for database type
 */
export function getDisplayName(dbType) {
  const displayNames = {
    [DatabaseType.POSTGRESQL]: "PostgreSQL",
    [DatabaseType.MSSQL]: "SQL Server",
    [DatabaseType.MYSQL]: "MySQL",
    [DatabaseType.MONGODB]: "MongoDB",
    [DatabaseType.REDIS]: "Redis",
    [DatabaseType.IGNITE]: "Apache Ignite",
    [DatabaseType.MARIADB]: "MariaDB",
    [DatabaseType.SQLITE]: "SQLite",
    [DatabaseType.ORACLE]: "Oracle",
    [DatabaseType.CASSANDRA]: "Cassandra",
  };
  return displayNames[dbType] || dbType;
}

/**
 * Get icon class for database type
 */
export function getIconClass(dbType) {
  const icons = {
    [DatabaseType.POSTGRESQL]: "fas fa-database",
    [DatabaseType.MSSQL]: "fas fa-database",
    [DatabaseType.MYSQL]: "fas fa-database",
    [DatabaseType.MONGODB]: "fas fa-leaf",
    [DatabaseType.REDIS]: "fas fa-cube",
    [DatabaseType.IGNITE]: "fas fa-fire",
    [DatabaseType.MARIADB]: "fas fa-database",
    [DatabaseType.SQLITE]: "fas fa-file-archive",
    [DatabaseType.ORACLE]: "fas fa-database",
    [DatabaseType.CASSANDRA]: "fas fa-server",
  };
  return icons[dbType] || "fas fa-database";
}

/**
 * Get primary object type name (tables, collections, keys, etc.)
 */
export function getPrimaryObjectType(dbType) {
  const objectTypes = {
    [DatabaseType.POSTGRESQL]: "tables",
    [DatabaseType.MSSQL]: "tables",
    [DatabaseType.MYSQL]: "tables",
    [DatabaseType.MONGODB]: "collections",
    [DatabaseType.REDIS]: "keys",
    [DatabaseType.IGNITE]: "caches",
    [DatabaseType.MARIADB]: "tables",
    [DatabaseType.SQLITE]: "tables",
    [DatabaseType.ORACLE]: "tables",
    [DatabaseType.CASSANDRA]: "tables",
  };
  return objectTypes[dbType] || "tables";
}

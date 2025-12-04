import { DatabaseType } from "../config/databaseTypes";

/**
 * Get default query template based on database type
 * @param {string} dbType - Type of database connection
 * @returns {string} Default query template
 */
export function getDefaultQuery(dbType) {
  const queries = {
    MySQL: `-- MySQL Query
SELECT * FROM table_name LIMIT 100;`,

    PostgreSQL: `-- PostgreSQL Query
SELECT * FROM table_name LIMIT 100;`,

    MSSQL: `-- SQL Server Query
SELECT TOP 100 * FROM table_name;`,

    MongoDB: `// MongoDB Query
db.collection_name.find({}).limit(100)`,

    Redis: `// Redis Commands
KEYS *
GET key_name
HGETALL hash_name`,

    Ignite: `-- Apache Ignite SCAN Query
-- Use SCAN to browse cache data (recommended)
SCAN cache_name LIMIT 100

-- Or use SQL if cache has SQL tables defined
-- SELECT * FROM "SCHEMA"."TABLE_NAME" LIMIT 100`,

    SQLite: `-- SQLite Query
SELECT * FROM table_name LIMIT 100;`,

    MariaDB: `-- MariaDB Query
SELECT * FROM table_name LIMIT 100;`,

    Oracle: `-- Oracle Query
SELECT * FROM table_name WHERE ROWNUM <= 100;`,
  };

  return queries[dbType] || `-- SQL Query\nSELECT * FROM table_name LIMIT 100;`;
}

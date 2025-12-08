/**
 * Query Formatter utility
 * Auto-formats SQL and MongoDB (JSON) queries based on database type
 */

import { formatSql } from "../sql/sqlFormatter";
import { formatJson } from "../json/jsonFormatter";
import { DatabaseType } from "../../../core/config/databaseTypes";

/**
 * Auto-format query berdasarkan database type
 * @param {string} query - Query string to format
 * @param {string} dbType - Database type (from DatabaseType enum)
 * @returns {string} - Formatted query, or original string if unable to format
 */
export function autoFormatQuery(query, dbType) {
  if (!query || typeof query !== "string") return query;

  try {
    // For MongoDB, try to format as JSON
    if (dbType === DatabaseType.MONGODB) {
      return formatJson(query);
    }

    // For SQL databases, format as SQL
    if (
      [
        DatabaseType.POSTGRESQL,
        DatabaseType.MSSQL,
        DatabaseType.MYSQL,
        DatabaseType.MARIADB,
        DatabaseType.SQLITE,
        DatabaseType.ORACLE,
      ].includes(dbType)
    ) {
      return formatSql(query);
    }

    // For unsupported database types, return original query
    return query;
  } catch (error) {
    console.error("Error auto-formatting query:", error);
    return query;
  }
}

export default autoFormatQuery;

import { format } from "sql-formatter";
import { DatabaseType } from "../../../core/config/databaseTypes";

/**
 * Simple SQL Formatter
 * Formats SQL queries dengan indentation dan line breaks yang lebih readable
 */

/**
 * Strip SQL comments dan semicolon dari query
 * @param {string} sql - SQL query
 * @returns {string} - Query tanpa comments dan semicolon
 */
export function stripSqlComments(sql, dbType) {
  if (!sql || typeof sql !== "string") return sql;

  let result = sql;

  // Remove multi-line comments /* ... */
  result = result.replace(/\/\*[\s\S]*?\*\//g, "");

  // Remove single-line comments -- ...
  result = result.replace(/--[^\n]*/g, "");

  // Remove hash comments # ... (untuk MySQL)
  if (dbType === DatabaseType.MYSQL) {
    result = result.replace(/#[^\n]*/g, "");
  }

  // Remove empty lines and trim, join with space
  result = result
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .join(" ");

  return result.trim();
}

export function formatSql(sql) {
  if (!sql || typeof sql !== "string") return sql;

  try {
    return format(sql, {
      language: "sql",
      indentStyle: "standard",
      keywordCase: "upper",
      linesBetweenQueries: 2,
    });
  } catch (error) {
    console.error("Error formatting SQL:", error);
    return sql;
  }
}

export default formatSql;

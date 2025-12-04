import * as monaco from "monaco-editor";
import {
  loadTablesForDatabase,
  loadColumnsForTable,
  generateTableAlias,
} from "./sqlEditorService";

/**
 * SQL Keywords for autocomplete
 */
const SQL_KEYWORDS = [
  "SELECT",
  "FROM",
  "WHERE",
  "INSERT",
  "UPDATE",
  "DELETE",
  "JOIN",
  "INNER",
  "LEFT",
  "RIGHT",
  "OUTER",
  "ON",
  "AND",
  "OR",
  "NOT",
  "IN",
  "EXISTS",
  "BETWEEN",
  "LIKE",
  "IS",
  "NULL",
  "ORDER",
  "BY",
  "GROUP",
  "HAVING",
  "LIMIT",
  "OFFSET",
  "AS",
  "DISTINCT",
  "ALL",
  "UNION",
  "INTERSECT",
  "EXCEPT",
  "CREATE",
  "ALTER",
  "DROP",
  "TABLE",
  "DATABASE",
  "INDEX",
  "VIEW",
  "PRIMARY",
  "KEY",
  "FOREIGN",
  "REFERENCES",
  "CONSTRAINT",
  "UNIQUE",
  "CHECK",
  "DEFAULT",
  "CASCADE",
  "SET",
  "VALUES",
  "INTO",
  "CASE",
  "WHEN",
  "THEN",
  "ELSE",
  "END",
  "COUNT",
  "SUM",
  "AVG",
  "MIN",
  "MAX",
  "CAST",
  "COALESCE",
  "NULLIF",
  "IFNULL",
  "CONCAT",
  "SUBSTRING",
  "TRIM",
  "UPPER",
  "LOWER",
  "LENGTH",
  "ROUND",
  "FLOOR",
  "CEIL",
  "ABS",
  "NOW",
  "CURDATE",
  "CURTIME",
  "DATE",
  "TIME",
  "DATETIME",
  "TIMESTAMP",
];

/**
 * Create Monaco completion provider for SQL
 */
export function createCompletionProvider(config) {
  const {
    selectedConn,
    selectedDb,
    databases,
    loadedDatabases,
    tableAliasMap,
  } = config;

  return {
    triggerCharacters: [" ", ".", "\n"],
    provideCompletionItems: async (model, position) => {
      const line = model.getLineContent(position.lineNumber);
      const textBefore = line.slice(0, position.column - 1);

      // Pre-load tables when user types database name (before dot)
      const dbNameMatch = textBefore.match(/\b(\w+)$/);
      if (dbNameMatch) {
        const typedWord = dbNameMatch[1];
        const matchedDb = databases.find(
          (db) => db.name.toLowerCase() === typedWord.toLowerCase()
        );
        if (matchedDb && !loadedDatabases[matchedDb.name]) {
          loadTablesForDatabase(
            selectedConn.id,
            matchedDb.name,
            loadedDatabases
          ).catch((err) =>
            console.error(`Background load failed for ${matchedDb.name}:`, err)
          );
        }
      }

      const suggestions = [];

      // Check for dot notation (table.column or database.table)
      const dotMatch = textBefore.match(/(\w+)\.(\w*)$/);
      if (dotMatch) {
        const [, identifier, partial] = dotMatch;

        // Check if we're in FROM/JOIN context for auto-alias
        const beforeDot = textBefore.substring(
          0,
          textBefore.lastIndexOf(identifier)
        );
        const inFromJoinContext = /(?:FROM|JOIN)\s+$/i.test(beforeDot);

        // Check if identifier is a database name (case-insensitive)
        const matchedDb = databases.find(
          (db) => db.name.toLowerCase() === identifier.toLowerCase()
        );

        if (matchedDb) {
          // Tables should already be loaded, just retrieve from cache
          const dbTables = await loadTablesForDatabase(
            selectedConn.id,
            matchedDb.name,
            loadedDatabases
          );
          suggestions.push(
            ...dbTables.map((table) => {
              // If in FROM/JOIN context, add auto-alias
              if (inFromJoinContext) {
                const alias = generateTableAlias(
                  table.name,
                  new Set(tableAliasMap.values())
                );
                return {
                  label: table.name,
                  kind: monaco.languages.CompletionItemKind.Class,
                  insertText: `${table.name} ${alias} `,
                  sortText: "1" + table.name,
                };
              }
              return {
                label: table.name,
                kind: monaco.languages.CompletionItemKind.Class,
                insertText: table.name,
                sortText: "1" + table.name,
              };
            })
          );
        } else if (selectedDb && loadedDatabases[selectedDb]?.tables) {
          const matchedTable = loadedDatabases[selectedDb].tables.find(
            (t) => t.name.toLowerCase() === identifier.toLowerCase()
          );
          if (matchedTable) {
            // Load columns for this table
            const columns = await loadColumnsForTable(
              selectedConn.id,
              selectedDb,
              matchedTable.name,
              loadedDatabases
            );
            suggestions.push(
              ...columns.map((col) => ({
                label: col,
                kind: monaco.languages.CompletionItemKind.Field,
                insertText: col,
                sortText: "1" + col,
              }))
            );
          }
        }

        // Check if identifier is an alias
        if (suggestions.length === 0 && tableAliasMap.has(identifier)) {
          const tableName = tableAliasMap.get(identifier);
          const dbName = selectedDb;
          if (dbName) {
            const columns = await loadColumnsForTable(
              selectedConn.id,
              dbName,
              tableName,
              loadedDatabases
            );
            suggestions.push(
              ...columns.map((col) => ({
                label: col,
                kind: monaco.languages.CompletionItemKind.Field,
                insertText: col,
                sortText: "1" + col,
              }))
            );
          }
        }

        return { suggestions };
      }

      // Check for FROM/JOIN context to show tables
      const fromMatch = textBefore.match(/(?:FROM|JOIN)\s+(\w*)$/i);
      if (fromMatch) {
        // If database is selected, load its tables
        if (selectedDb) {
          const dbTables = await loadTablesForDatabase(
            selectedConn.id,
            selectedDb,
            loadedDatabases
          );
          suggestions.push(
            ...dbTables.map((table) => {
              const alias = generateTableAlias(
                table.name,
                new Set(tableAliasMap.values())
              );
              return {
                label: table.name,
                kind: monaco.languages.CompletionItemKind.Class,
                insertText: `${table.name} ${alias} `,
                sortText: "2" + table.name,
              };
            })
          );
        }

        // Also show other databases
        suggestions.push(
          ...databases.map((db) => ({
            label: db.name,
            kind: monaco.languages.CompletionItemKind.Module,
            insertText: db.name,
            detail: "database",
            sortText: "3" + db.name,
          }))
        );

        return { suggestions };
      }

      // Add SQL keywords
      suggestions.push(
        ...SQL_KEYWORDS.map((keyword) => ({
          label: keyword,
          kind: monaco.languages.CompletionItemKind.Keyword,
          insertText: keyword + " ",
          sortText: "0" + keyword,
        }))
      );

      // Add tables from current database
      if (selectedDb && loadedDatabases[selectedDb]) {
        const dbTables = loadedDatabases[selectedDb].tables || [];
        suggestions.push(
          ...dbTables.map((table) => ({
            label: table.name,
            kind: monaco.languages.CompletionItemKind.Class,
            insertText: table.name,
            sortText: "4" + table.name,
          }))
        );
      }

      // Add databases
      suggestions.push(
        ...databases.map((db) => ({
          label: db.name,
          kind: monaco.languages.CompletionItemKind.Module,
          insertText: db.name,
          detail: "database",
          sortText: "5" + db.name,
        }))
      );

      return { suggestions };
    },
  };
}

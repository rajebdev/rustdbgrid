/**
 * Grid Edit Service
 * Handles cell editing, row updates, and SQL generation
 */

/**
 * Start editing a cell
 */
export function startEdit(rowIndex, column, currentValue) {
  let valueStr;
  if (currentValue === null || currentValue === undefined) {
    valueStr = "";
  } else if (Array.isArray(currentValue)) {
    valueStr = JSON.stringify(currentValue);
  } else if (typeof currentValue === "object") {
    valueStr = JSON.stringify(currentValue);
  } else {
    valueStr = String(currentValue);
  }

  // Check if value is long or has newlines
  const estimatedWidth = valueStr.length * 7;
  const hasNewlines = valueStr.includes("\n") || valueStr.includes("\r");

  return {
    shouldUsePopupEditor:
      estimatedWidth > 500 || hasNewlines || valueStr.length > 70,
    valueStr,
    originalValue: currentValue,
  };
}

/**
 * Track edits in the editedRows map
 */
export function trackEditedRow(
  rowIndex,
  column,
  newValue,
  displayData,
  editedRows,
  originalRowData
) {
  // Backup original row data if first edit on this row
  if (!originalRowData.has(rowIndex)) {
    originalRowData.set(rowIndex, { ...displayData.rows[rowIndex] });
  }

  // Store edit
  if (!editedRows.has(rowIndex)) {
    editedRows.set(rowIndex, new Map());
  }
  editedRows.get(rowIndex).set(column, newValue);

  // Update display
  displayData.rows[rowIndex][column] = newValue;
  displayData = { ...displayData };

  // Force reactivity
  editedRows = new Map(editedRows);

  return { displayData, editedRows };
}

/**
 * Cancel all edits and restore original data
 */
export function cancelAllEdits(displayData, editedRows, originalRowData) {
  console.log("🔄 Canceling all edits, restoring original data...");

  // Restore original values
  originalRowData.forEach((originalRow, rowIndex) => {
    console.log(`  Restoring row ${rowIndex}:`, originalRow);
    displayData.rows[rowIndex] = { ...originalRow };
  });

  // Force reactivity
  displayData = { ...displayData };

  // Clear edits and backups
  editedRows.clear();
  editedRows = new Map();
  originalRowData.clear();
  originalRowData = new Map();

  console.log("✅ All edits canceled, data restored");

  return { displayData, editedRows, originalRowData };
}

/**
 * Generate UPDATE SQL statements from edits
 */
export function generateUpdateSql(displayData, editedRows, executedQuery) {
  if (!displayData || !displayData.rows || !executedQuery) {
    return [];
  }

  const updates = [];

  // Extract table name from query
  let tableMatch = executedQuery.match(/FROM\s+`?(\w+)`?\.`?(\w+)`?/i);
  let tableName = "";

  if (tableMatch) {
    tableName = `\`${tableMatch[1]}\`.\`${tableMatch[2]}\``;
    console.log("📋 Extracted table name (with schema):", tableName);
  } else {
    tableMatch = executedQuery.match(/FROM\s+`?(\w+)`?/i);
    tableName = tableMatch ? `\`${tableMatch[1]}\`` : "`table`";
    console.log("📋 Extracted table name (simple):", tableName);
  }

  console.log("📊 Available column_types:", displayData.column_types);

  editedRows.forEach((changes, rowIndex) => {
    const row = displayData.rows[rowIndex];
    if (!row) return;

    const setClauses = [];
    const whereClauses = [];

    // Build SET clause
    changes.forEach((newValue, column) => {
      const sqlValue =
        newValue === "" ? "NULL" : `'${newValue.replace(/'/g, "''")}'`;
      setClauses.push(`\`${column}\` = ${sqlValue}`);
    });

    // Build WHERE clause
    displayData.columns.forEach((column, columnIndex) => {
      if (!column) return;
      const columnName = typeof column === "object" ? column.name : column;

      if (!changes.has(columnName)) {
        // Handle both array and object row formats
        const value = Array.isArray(row) ? row[columnIndex] : row[columnName];

        if (value !== null && value !== undefined) {
          const columnType =
            displayData.column_types?.[columnName]?.toUpperCase() || "";
          let sqlValue;

          if (
            columnType.includes("INT") ||
            columnType.includes("DECIMAL") ||
            columnType.includes("NUMERIC") ||
            columnType.includes("FLOAT") ||
            columnType.includes("DOUBLE") ||
            columnType.includes("REAL")
          ) {
            sqlValue = value;
            console.log(`  ${columnName}: ${value} (${columnType}, no quotes)`);
          } else if (
            columnType.includes("BOOL") ||
            columnType.includes("BIT")
          ) {
            sqlValue = value ? 1 : 0;
            console.log(
              `  ${columnName}: ${value} (${columnType} -> ${sqlValue})`
            );
          } else if (
            columnType.includes("DATE") ||
            columnType.includes("TIME") ||
            columnType.includes("TIMESTAMP")
          ) {
            sqlValue = `'${String(value).replace(/'/g, "''")}'`;
            console.log(`  ${columnName}: "${value}" (${columnType}, quoted)`);
          } else {
            sqlValue = `'${String(value).replace(/'/g, "''")}'`;
            console.log(`  ${columnName}: "${value}" (${columnType}, quoted)`);
          }

          whereClauses.push(`\`${columnName}\` = ${sqlValue}`);
        }
      }
    });

    if (setClauses.length > 0 && whereClauses.length > 0) {
      const sql = `UPDATE ${tableName} SET ${setClauses.join(
        ", "
      )} WHERE ${whereClauses.join(" AND ")};`;
      console.log("✅ Generated SQL:", sql);
      updates.push({
        sql,
        rowIndex,
      });
    }
  });

  return updates;
}

/**
 * Check if there are unsaved edits
 */
export function hasEdits(editedRows) {
  return editedRows.size > 0;
}

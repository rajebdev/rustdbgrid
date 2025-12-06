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
 * Convert row data (array or object) to object format
 * @param {Array|Object} row - Row data that could be array or object
 * @param {Array} columnNames - Column names (required if row is array)
 * @returns {Object} Row data as object
 */
function rowToObject(row, columnNames = []) {
  if (Array.isArray(row)) {
    // Convert array to object using column names
    const obj = {};
    row.forEach((value, index) => {
      if (columnNames[index]) {
        obj[columnNames[index]] = value;
      }
    });
    return obj;
  }
  return { ...row };
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
  originalRowData,
  columnNames = []
) {
  // Get column names from displayData if not provided
  let colNames = columnNames || displayData.column_names || [];
  if (colNames.length === 0 && displayData.columns) {
    // Extract column names from columns array
    colNames = displayData.columns.map((col) => col.name);
  }

  // Backup original row data if first edit on this row
  if (!originalRowData.has(rowIndex)) {
    const rowData = displayData.rows[rowIndex];
    const originalAsObject = rowToObject(rowData, colNames);
    originalRowData.set(rowIndex, originalAsObject);
    console.log(`[trackEditedRow] Backing up row ${rowIndex}:`, {
      rowData,
      columnNames: colNames,
      originalAsObject,
    });
  }

  // Store edit
  if (!editedRows.has(rowIndex)) {
    editedRows.set(rowIndex, new Map());
  }
  editedRows.get(rowIndex).set(column, newValue);

  // Update display
  if (Array.isArray(displayData.rows[rowIndex])) {
    const columnIndex = colNames.indexOf(column);
    if (columnIndex >= 0) {
      displayData.rows[rowIndex][columnIndex] = newValue;
    }
  } else {
    displayData.rows[rowIndex][column] = newValue;
  }
  displayData = { ...displayData };

  // Force reactivity
  editedRows = new Map(editedRows);

  return { displayData, editedRows };
}

/**
 * Cancel all edits and restore original data
 */
export function cancelAllEdits(displayData, editedRows, originalRowData) {
  console.log("[cancelAllEdits service] Starting restore:", {
    originalRowDataSize: originalRowData.size,
    displayDataRowsCount: displayData.rows.length,
  });

  // Restore original values
  originalRowData.forEach((originalRow, rowIndex) => {
    const currentRow = displayData.rows[rowIndex];

    console.log(`[cancelAllEdits] Restoring row ${rowIndex}:`, {
      isArray: Array.isArray(currentRow),
      originalRow,
    });

    if (Array.isArray(currentRow)) {
      // If current row is array, restore from object back to array format
      // Get column names from displayData
      let colNames = displayData.column_names || [];
      if (colNames.length === 0 && displayData.columns) {
        colNames = displayData.columns.map((col) => col.name);
      }

      // Convert object back to array
      const restoredArray = colNames.map((colName) => originalRow[colName]);
      displayData.rows[rowIndex] = restoredArray;
      console.log(
        `[cancelAllEdits] Restored array row ${rowIndex}:`,
        restoredArray
      );
    } else {
      // If current row is object, just restore object
      displayData.rows[rowIndex] = { ...originalRow };
      console.log(`[cancelAllEdits] Restored object row ${rowIndex}:`);
    }
  });

  // Restore deleted rows (remove _isDeleted flag from all rows)
  displayData.rows = displayData.rows.map((row) => {
    if (Array.isArray(row)) {
      // For array rows, remove non-enumerable _isDeleted property
      if (Object.getOwnPropertyDescriptor(row, "_isDeleted")) {
        delete row._isDeleted;
      }
      return row;
    } else if (typeof row === "object" && row._isDeleted) {
      // For object rows, remove _isDeleted property
      const cleanRow = { ...row };
      delete cleanRow._isDeleted;
      return cleanRow;
    }
    return row;
  });

  // Force reactivity
  displayData = { ...displayData };

  // Clear edits and backups
  editedRows.clear();
  editedRows = new Map();
  originalRowData.clear();
  originalRowData = new Map();

  console.log("✅ All edits and deletes canceled, data restored");

  return { displayData, editedRows, originalRowData };
}

/**
 * Check if there are unsaved edits
 */
export function hasEdits(editedRows) {
  return editedRows.size > 0;
}

/**
 * Grid Row Service
 * Handles row operations: add, delete, duplicate
 */

/**
 * Add a new empty row to displayData
 * @param {Array} displayData - Current display data
 * @param {Array} columns - Column definitions
 * @param {Map} newRows - Map of new rows
 * @param {number} insertAfterIndex - Index to insert after (optional, default appends at end)
 * @returns {Object} { displayData, newRows }
 */
export function addNewRow(
  displayData,
  columns,
  newRows,
  insertAfterIndex = null
) {
  if (!displayData || !columns) {
    console.error("Invalid input to addNewRow");
    return { displayData, newRows };
  }

  // Create empty row object with null values
  const emptyRow = {};
  columns.forEach((col) => {
    const colName = typeof col === "object" ? col.name : col;
    emptyRow[colName] = null;
  });

  // Generate a unique ID for the new row
  const newRowId = `new_row_${Date.now()}_${Math.random()}`;

  // Add to newRows map with the empty data
  newRows.set(newRowId, { ...emptyRow });

  // Add to displayData with _isNewRow metadata and _rowId
  const newRowData = { ...emptyRow, _rowId: newRowId, _isNewRow: true };

  let updatedDisplayData;
  if (
    insertAfterIndex !== null &&
    insertAfterIndex >= 0 &&
    insertAfterIndex < displayData.length
  ) {
    // Insert after the specified index
    updatedDisplayData = [
      ...displayData.slice(0, insertAfterIndex + 1),
      newRowData,
      ...displayData.slice(insertAfterIndex + 1),
    ];
  } else {
    // Append at the end
    updatedDisplayData = [...displayData, newRowData];
  }

  return { displayData: updatedDisplayData, newRows };
}

/**
 * Delete a row from displayData and mark it in deletedRows
 * @param {Array} displayData - Current display data
 * @param {number} rowIndex - Index of row to delete
 * @param {Set} deletedRows - Set of deleted row data (not indices)
 * @param {Map} editedRows - Map of edited rows
 * @param {Map} newRows - Map of new rows (to remove if it's a new row)
 * @returns {Object} { displayData, deletedRows, editedRows, newRows }
 */
export function deleteRow(
  displayData,
  rowIndex,
  deletedRows,
  editedRows,
  newRows,
  columns
) {
  if (!displayData || rowIndex < 0 || rowIndex >= displayData.length) {
    console.error("Invalid input to deleteRow");
    return { displayData, deletedRows, editedRows, newRows };
  }

  const row = displayData[rowIndex];
  const isNewRow =
    row._isNewRow === true ||
    (row._rowId && String(row._rowId).startsWith("new_row_"));

  // If it's a new row, just remove from display and newRows Map
  if (isNewRow) {
    if (row._rowId && newRows.has(row._rowId)) {
      newRows.delete(row._rowId);
    }
    const updatedDisplayData = displayData.filter((_, i) => i !== rowIndex);
    return {
      displayData: updatedDisplayData,
      deletedRows,
      editedRows,
      newRows,
    };
  }

  // For existing rows, convert to object with proper column names if needed
  let cleanRow = {};

  console.log("[deleteRow] Raw row:", row);
  console.log("[deleteRow] Is array?", Array.isArray(row));
  console.log("[deleteRow] Columns:", columns);

  if (Array.isArray(row)) {
    // If array, convert to object using columns
    if (!columns || columns.length === 0) {
      console.error(
        "Cannot delete array row without column metadata. Columns:",
        columns
      );
      return { displayData, deletedRows, editedRows, newRows };
    }
    columns.forEach((col, index) => {
      const colName = typeof col === "object" ? col.name : col;
      cleanRow[colName] = row[index];
    });
    console.log("[deleteRow] Converted array row to object:", cleanRow);
  } else {
    // If object, copy it
    cleanRow = { ...row };
    console.log("[deleteRow] Copied object row:", cleanRow);
  }

  // Remove metadata
  delete cleanRow._rowId;
  delete cleanRow._isNewRow;
  delete cleanRow._isEdited;
  delete cleanRow._isDeleted;

  console.log("[deleteRow] Final row to delete:", cleanRow);
  console.log("[deleteRow] Final row keys:", Object.keys(cleanRow));

  deletedRows.add(cleanRow);

  // Remove from edited rows if it was edited
  if (editedRows.has(rowIndex)) {
    editedRows.delete(rowIndex);
  }

  // Mark row as deleted in display (add _isDeleted flag)
  // Handle both array and object rows
  const updatedDisplayData = displayData.map((r, i) => {
    if (i === rowIndex) {
      if (Array.isArray(r)) {
        // For array rows, can't add property directly - keep as array
        // Mark with non-enumerable property
        Object.defineProperty(r, "_isDeleted", {
          value: true,
          enumerable: false,
          configurable: true,
        });
        return r;
      } else {
        // For object rows, add property normally
        return { ...r, _isDeleted: true };
      }
    }
    return r;
  });

  return {
    displayData: updatedDisplayData,
    deletedRows: new Set(deletedRows),
    editedRows,
    newRows,
  };
}

/**
 * Duplicate a row
 * @param {Array} displayData - Current display data
 * @param {number} rowIndex - Index of row to duplicate
 * @param {Map} newRows - Map of new rows
 * @param {Map} editedRows - Map of edited rows
 * @param {Array} columns - Column definitions with metadata (including is_auto_increment)
 * @returns {Object} { displayData, newRows }
 */
export function duplicateRow(
  displayData,
  rowIndex,
  newRows,
  editedRows,
  columns
) {
  if (!displayData || rowIndex < 0 || rowIndex >= displayData.length) {
    console.error("Invalid input to duplicateRow");
    return { displayData, newRows };
  }

  const rowToDuplicate = displayData[rowIndex];
  if (!rowToDuplicate) {
    console.error("Row to duplicate is null or undefined");
    return { displayData, newRows };
  }

  // Convert row to object format with proper column names
  let duplicatedRow = {};

  if (Array.isArray(rowToDuplicate)) {
    // If array, convert to object using columns
    if (!columns || columns.length === 0) {
      console.error("Cannot duplicate array row without column metadata");
      return { displayData, newRows };
    }
    columns.forEach((col, index) => {
      const colName = typeof col === "object" ? col.name : col;
      duplicatedRow[colName] = rowToDuplicate[index];
    });
  } else {
    // If already object, copy it (excluding metadata)
    duplicatedRow = { ...rowToDuplicate };
    // Remove metadata
    delete duplicatedRow._rowId;
    delete duplicatedRow._isEdited;
    delete duplicatedRow._isNewRow;
    delete duplicatedRow._isDeleted;
  }

  console.log("[duplicateRow] Row to duplicate:", rowToDuplicate);
  console.log("[duplicateRow] Converted to object:", duplicatedRow);

  // Set auto-increment columns to NULL
  if (columns && columns.length > 0) {
    columns.forEach((col) => {
      const colName = typeof col === "object" ? col.name : col;
      // Check if column has is_auto_increment property and it's true
      if (typeof col === "object" && col.is_auto_increment === true) {
        duplicatedRow[colName] = null;
        console.log(
          `[duplicateRow] Set auto-increment column '${colName}' to NULL`
        );
      }
    });
  }

  // Generate a unique ID for the new row
  const newRowId = `new_row_${Date.now()}_${Math.random()}`;

  // Add to newRows map with the data
  newRows.set(newRowId, JSON.parse(JSON.stringify(duplicatedRow))); // Deep copy

  // Create new row data with metadata
  const newRowData = { ...duplicatedRow, _rowId: newRowId, _isNewRow: true };

  // Add to displayData right after the original row
  const updatedDisplayData = [
    ...displayData.slice(0, rowIndex + 1),
    newRowData,
    ...displayData.slice(rowIndex + 1),
  ];

  console.log("[duplicateRow] New row created with data:", newRowData);
  return { displayData: updatedDisplayData, newRows };
}

/**
 * Get column names from displayData
 * @param {Array} displayData - Current display data
 * @returns {Array} Array of column names
 */
export function getColumnsFromData(displayData) {
  if (!displayData || displayData.length === 0) {
    return [];
  }

  const firstRow = displayData[0];
  return Object.keys(firstRow).filter((key) => !key.startsWith("_"));
}

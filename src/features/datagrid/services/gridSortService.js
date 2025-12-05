/**
 * Grid Sort Service
 * Handles multi-column sorting logic with support for ASC, DESC, and NONE states
 */

/**
 * Handle sort column click with multi-column support
 * - Single click: cycles through ASC → DESC → NONE
 * - Ctrl+Click: adds column to sort stack
 * - Shift+Click: removes column from sort stack
 */
export function handleSort(
  column,
  sortStack,
  tabId,
  tabDataStore,
  onSortChanged,
  isCtrlPressed = false,
  isShiftPressed = false
) {
  let newSortStack = [...sortStack];
  const existingIndex = newSortStack.findIndex((s) => s.column === column);

  console.log(
    `[GridSortService] handleSort called - column: ${column}, existingIndex: ${existingIndex}, Ctrl: ${isCtrlPressed}, Shift: ${isShiftPressed}`
  );

  // Shift+Click: Remove from sort stack
  if (isShiftPressed) {
    if (existingIndex >= 0) {
      console.log(
        `[GridSortService] Shift+Click: Removing "${column}" from sort stack`
      );
      newSortStack.splice(existingIndex, 1);
      // Re-assign priorities
      newSortStack = newSortStack.map((s, idx) => ({
        ...s,
        priority: idx + 1,
      }));
    }
  }
  // Ctrl+Click: Add to sort stack (or keep if already exists)
  else if (isCtrlPressed) {
    if (existingIndex < 0) {
      // Add new column to sort stack
      const maxPriority = Math.max(0, ...newSortStack.map((s) => s.priority));
      console.log(
        `[GridSortService] Ctrl+Click: Adding "${column}" to sort stack with priority ${
          maxPriority + 1
        }`
      );
      newSortStack.push({
        column,
        direction: "asc",
        priority: maxPriority + 1,
      });
    }
    // If already in stack, do nothing (just focus the column)
  }
  // Single click: Cycle through ASC → DESC → NONE
  else {
    if (existingIndex >= 0) {
      const current = newSortStack[existingIndex];
      if (current.direction === "asc") {
        // ASC → DESC
        console.log(
          `[GridSortService] Single click: Cycling ASC → DESC for "${column}"`
        );
        newSortStack[existingIndex].direction = "desc";
      } else {
        // DESC → NONE (remove from stack)
        console.log(
          `[GridSortService] Single click: Cycling DESC → NONE (removing) for "${column}"`
        );
        newSortStack.splice(existingIndex, 1);
        // Re-assign priorities
        newSortStack = newSortStack.map((s, idx) => ({
          ...s,
          priority: idx + 1,
        }));
      }
    } else {
      // Not in stack, add it with ASC
      console.log(
        `[GridSortService] Single click: Adding "${column}" to sort stack (first time)`
      );
      const maxPriority = Math.max(0, ...newSortStack.map((s) => s.priority));
      newSortStack.push({
        column,
        direction: "asc",
        priority: maxPriority + 1,
      });
    }
  }

  console.log(`[GridSortService] Result stack:`, newSortStack);

  // Save to store
  if (tabId) {
    tabDataStore.setSortStack(tabId, newSortStack);
  }

  // Trigger callback
  if (onSortChanged) {
    onSortChanged();
  }

  return {
    sortStack: newSortStack,
  };
}

/**
 * Get sort info for a specific column
 * @returns {null|Object} - null if not sorted, or { direction, priority }
 */
export function getColumnSortInfo(column, sortStack) {
  const sort = sortStack.find((s) => s.column === column);
  return sort
    ? {
        direction: sort.direction,
        priority: sort.priority,
      }
    : null;
}

/**
 * Clear all sorts
 */
export function clearAllSorts(tabId, tabDataStore, onSortChanged) {
  if (tabId) {
    tabDataStore.setSortStack(tabId, []);
  }

  if (onSortChanged) {
    onSortChanged();
  }

  return {
    sortStack: [],
  };
}

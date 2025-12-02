import { writable, derived } from "svelte/store";

/**
 * Composable for search/filter functionality
 * @param {import('svelte/store').Readable} itemsStore - Svelte store containing items
 * @param {string[]} searchFields - Fields to search in (supports nested with dot notation)
 * @returns {Object} Search term store and filtered items store
 */
export function useSearch(itemsStore, searchFields) {
  const searchTerm = writable("");

  const filteredItems = derived(
    [itemsStore, searchTerm],
    ([$items, $searchTerm]) => {
      if (!$searchTerm || !$searchTerm.trim()) {
        return $items;
      }

      const term = $searchTerm.toLowerCase();

      return $items.filter((item) =>
        searchFields.some((field) => {
          // Support nested fields like "user.name"
          const value = field.split(".").reduce((obj, key) => obj?.[key], item);
          return value && String(value).toLowerCase().includes(term);
        })
      );
    }
  );

  return {
    searchTerm,
    filteredItems,
  };
}

/**
 * Simple writable store version for non-reactive arrays
 * @param {Array} items - Initial items array
 * @param {string[]} searchFields - Fields to search in
 * @returns {Object} Search functions and state
 */
export function createSearchFilter(items, searchFields) {
  let currentTerm = "";
  let currentItems = items;

  function filter(term) {
    currentTerm = term;
    if (!term || !term.trim()) {
      return currentItems;
    }

    const searchTerm = term.toLowerCase();
    return currentItems.filter((item) =>
      searchFields.some((field) => {
        const value = field.split(".").reduce((obj, key) => obj?.[key], item);
        return value && String(value).toLowerCase().includes(searchTerm);
      })
    );
  }

  function updateItems(newItems) {
    currentItems = newItems;
    return filter(currentTerm);
  }

  return {
    filter,
    updateItems,
    getTerm: () => currentTerm,
  };
}

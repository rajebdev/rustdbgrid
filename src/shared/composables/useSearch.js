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

/**
 * Debounce Utility
 * Creates debounced functions that delay execution until after a specified time
 */

/**
 * Create a debounced version of a function
 * @param {Function} fn - The function to debounce
 * @param {number} delay - Delay in milliseconds
 * @returns {Object} Object with trigger, cleanup, and immediate functions
 */
export function createDebounce(fn, delay) {
  let timeoutId = null;

  return {
    /**
     * Trigger the debounced function
     * Cancels previous calls and schedules a new execution
     */
    trigger: (...args) => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      timeoutId = setTimeout(() => fn(...args), delay);
    },

    /**
     * Execute the function immediately, bypassing debounce
     */
    immediate: (...args) => {
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeoutId = null;
      }
      return fn(...args);
    },

    /**
     * Cancel any pending execution
     */
    cleanup: () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeoutId = null;
      }
    },
  };
}

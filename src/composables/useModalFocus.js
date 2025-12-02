/**
 * Auto-focus and select text in an input or textarea element
 * @param {HTMLElement} node - The input or textarea element
 */
export function autoFocus(node) {
  node.focus();
  node.select();
}

/**
 * Just focus an element without selecting
 * @param {HTMLElement} node - The element to focus
 */
export function focusElement(node) {
  node.focus();
}

/**
 * Focus an element with a delay (useful for modals that need to render first)
 * @param {HTMLElement} node - The element to focus
 * @param {number} delay - Delay in milliseconds
 */
export function delayedFocus(node, delay = 100) {
  setTimeout(() => {
    node.focus();
  }, delay);
}

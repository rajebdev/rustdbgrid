/**
 * Clipboard Utility
 * Provides a unified interface for copying text to clipboard
 */

/**
 * Copy text to clipboard with optional success message
 * @param {string} text - The text to copy to clipboard
 * @param {string} [successMsg] - Optional success message to log
 * @returns {Promise<boolean>} True if successful, false otherwise
 */
export async function copyToClipboard(text, successMsg = null) {
  try {
    await navigator.clipboard.writeText(text);
    if (successMsg) {
      console.log(successMsg);
    }
    return true;
  } catch (err) {
    console.error("Failed to copy to clipboard:", err);
    return false;
  }
}

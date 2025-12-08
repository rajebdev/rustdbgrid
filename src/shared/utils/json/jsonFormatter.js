/**
 * JSON Formatter utility
 * Formats JSON strings with proper indentation and validation
 */

/**
 * Format JSON string dengan proper indentation
 * @param {string} jsonStr - JSON string to format
 * @param {number} indentSize - Indentation size (default: 2)
 * @returns {string} - Formatted JSON string, or original string if invalid JSON
 */
export function formatJson(jsonStr, indentSize = 2) {
  if (!jsonStr || typeof jsonStr !== "string") return jsonStr;

  try {
    // Try to parse the JSON string
    const parsed = JSON.parse(jsonStr);
    // Format with proper indentation
    return JSON.stringify(parsed, null, indentSize);
  } catch (error) {
    console.error("Error formatting JSON:", error);
    // Return original string if not valid JSON
    return jsonStr;
  }
}

/**
 * Validate if string is valid JSON
 * @param {string} jsonStr - JSON string to validate
 * @returns {boolean} - True if valid JSON, false otherwise
 */
export function isValidJson(jsonStr) {
  if (!jsonStr || typeof jsonStr !== "string") return false;

  try {
    JSON.parse(jsonStr);
    return true;
  } catch (error) {
    return false;
  }
}

export default formatJson;

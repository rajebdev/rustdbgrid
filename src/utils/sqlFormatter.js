/**
 * Simple SQL Formatter
 * Formats SQL queries dengan indentation dan line breaks yang lebih readable
 */

function formatSqlSimple(sql) {
  if (!sql || typeof sql !== "string") return sql;

  let formatted = sql;

  // Keywords yang harus diikuti newline
  const keywordsNewline = [
    "SELECT",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "NOT",
    "JOIN",
    "INNER JOIN",
    "LEFT JOIN",
    "RIGHT JOIN",
    "FULL JOIN",
    "CROSS JOIN",
    "ON",
    "GROUP BY",
    "ORDER BY",
    "HAVING",
    "LIMIT",
    "OFFSET",
    "UNION",
    "EXCEPT",
    "INTERSECT",
    "INSERT INTO",
    "UPDATE",
    "DELETE FROM",
    "CREATE",
    "ALTER",
    "DROP",
    "WITH",
  ];

  // Replace multiple spaces/newlines dengan single space
  formatted = formatted.replace(/\s+/g, " ").trim();

  // Add newlines sebelum keywords (case-insensitive)
  keywordsNewline.forEach((keyword) => {
    const regex = new RegExp(`\\s+(${keyword})\\s+`, "gi");
    formatted = formatted.replace(regex, `\n${keyword} `);
  });

  // Handle commas dalam SELECT (tambah newline setelah comma untuk columns)
  formatted = formatted.replace(/,\s+/g, ",\n  ");

  // Add indentation
  const lines = formatted.split("\n");
  const indented = lines
    .map((line) => {
      line = line.trim();
      if (!line) return "";

      // Keywords utama tanpa indent
      if (
        /^(SELECT|FROM|WHERE|GROUP|ORDER|UNION|INTERSECT|EXCEPT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|WITH)/.test(
          line
        )
      ) {
        return line;
      }
      // Indentasi untuk bagian lain
      return "  " + line;
    })
    .filter((line) => line.trim());

  return indented.join("\n");
}

export function formatSql(sql, fullText = false) {
  if (!sql || typeof sql !== "string") return sql;

  if (fullText) {
    return formatSqlSimple(sql);
  }

  // Untuk selected text, lebih simple formatting
  let formatted = sql.trim();

  // Replace multiple spaces dengan single space
  formatted = formatted.replace(/\s+/g, " ");

  // Simple keywords formatting untuk selected text
  const keywords = [
    "SELECT",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "NOT",
    "JOIN",
    "ON",
    "GROUP BY",
    "ORDER BY",
    "LIMIT",
  ];

  keywords.forEach((keyword) => {
    const regex = new RegExp(`\\b${keyword}\\b`, "gi");
    formatted = formatted.replace(regex, keyword.toUpperCase());
  });

  return formatted;
}

export default formatSql;

/**
 * Sinh HTML đã tô màu (syntax highlight) cho một câu lệnh SQL.
 *
 * Dùng cho lớp phủ highlight phía sau `<textarea>`: mỗi loại token được bọc trong
 * `<span>` với class riêng (`sql-kw`, `sql-str`, `sql-com`, `sql-num`), phần còn lại
 * được escape HTML. Nội dung text (kể cả khoảng trắng/xuống dòng) khớp 1-1 với
 * textarea để hai lớp căn khít.
 */

/** Từ khoá SQL (một từ) được tô màu. */
const KEYWORDS = new Set([
  "SELECT", "FROM", "WHERE", "GROUP", "ORDER", "BY", "HAVING", "LIMIT", "OFFSET",
  "JOIN", "INNER", "LEFT", "RIGHT", "FULL", "CROSS", "OUTER", "ON", "USING",
  "AS", "AND", "OR", "NOT", "IN", "IS", "NULL", "LIKE", "ILIKE", "BETWEEN",
  "EXISTS", "ALL", "ANY", "DISTINCT", "UNION", "EXCEPT", "INTERSECT",
  "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "RETURNING",
  "CREATE", "ALTER", "DROP", "TABLE", "VIEW", "INDEX", "CASE", "WHEN", "THEN",
  "ELSE", "END", "ASC", "DESC", "TRUE", "FALSE", "WITH", "OVER", "PARTITION",
  "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "CAST", "NULLIF",
]);

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function span(cls: string, text: string): string {
  return `<span class="${cls}">${escapeHtml(text)}</span>`;
}

export function highlightSql(sql: string): string {
  let out = "";
  let i = 0;
  const n = sql.length;

  while (i < n) {
    const ch = sql[i];

    // Comment dòng: -- ... hết dòng.
    if (ch === "-" && sql[i + 1] === "-") {
      let j = i + 2;
      while (j < n && sql[j] !== "\n") j += 1;
      out += span("sql-com", sql.slice(i, j));
      i = j;
      continue;
    }

    // Comment khối: /* ... */
    if (ch === "/" && sql[i + 1] === "*") {
      let j = i + 2;
      while (j < n && !(sql[j] === "*" && sql[j + 1] === "/")) j += 1;
      j = Math.min(n, j + 2);
      out += span("sql-com", sql.slice(i, j));
      i = j;
      continue;
    }

    // Chuỗi 'literal' (escape '' bên trong).
    if (ch === "'") {
      let j = i + 1;
      while (j < n) {
        if (sql[j] === "'" && sql[j + 1] === "'") {
          j += 2;
          continue;
        }
        if (sql[j] === "'") {
          j += 1;
          break;
        }
        j += 1;
      }
      out += span("sql-str", sql.slice(i, j));
      i = j;
      continue;
    }

    // Định danh có ngoặc kép "id".
    if (ch === '"') {
      let j = i + 1;
      while (j < n && sql[j] !== '"') j += 1;
      j = Math.min(n, j + 1);
      out += span("sql-str", sql.slice(i, j));
      i = j;
      continue;
    }

    // Số.
    if (/[0-9]/.test(ch)) {
      let j = i;
      while (j < n && /[0-9.]/.test(sql[j])) j += 1;
      out += span("sql-num", sql.slice(i, j));
      i = j;
      continue;
    }

    // Từ / định danh — tô màu nếu là từ khoá.
    if (/[A-Za-z_]/.test(ch)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_$]/.test(sql[j])) j += 1;
      const word = sql.slice(i, j);
      out += KEYWORDS.has(word.toUpperCase()) ? span("sql-kw", word) : escapeHtml(word);
      i = j;
      continue;
    }

    // Khoảng trắng / dấu câu / toán tử — giữ nguyên (đã escape).
    out += escapeHtml(ch);
    i += 1;
  }

  return out;
}

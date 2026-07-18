/**
 * Bộ format (beautify) SQL gọn nhẹ, không phụ thuộc thư viện ngoài.
 *
 * Cách tiếp cận: tokenize câu lệnh (giữ nguyên chuỗi/comment như một token) rồi
 * dựng lại với quy tắc xuống dòng + thụt lề cơ bản. Vì output được dựng lại từ
 * token nên không bao giờ làm mất nội dung — chỉ thay đổi khoảng trắng và hoa/thường
 * của từ khoá.
 */

/** Từ khoá bắt đầu một mệnh đề chính → xuống dòng mới ở mức thụt lề hiện tại. */
const CLAUSE_KEYWORDS = new Set([
  "SELECT",
  "FROM",
  "WHERE",
  "HAVING",
  "LIMIT",
  "OFFSET",
  "UNION",
  "EXCEPT",
  "INTERSECT",
  "VALUES",
  "SET",
  "RETURNING",
  "INSERT",
  "UPDATE",
  "DELETE",
]);

/** Cụm từ khoá 2 từ cũng bắt đầu một mệnh đề chính. */
const CLAUSE_PHRASES = new Set([
  "GROUP BY",
  "ORDER BY",
  "INSERT INTO",
  "DELETE FROM",
  "UNION ALL",
  "CROSS JOIN",
  "INNER JOIN",
  "LEFT JOIN",
  "RIGHT JOIN",
  "FULL JOIN",
  "LEFT OUTER JOIN",
  "RIGHT OUTER JOIN",
  "FULL OUTER JOIN",
]);

/** Từ khoá join đơn + `ON` → xuống dòng. */
const JOIN_KEYWORDS = new Set(["JOIN", "ON"]);

/** `AND` / `OR` → xuống dòng, thụt thêm một mức so với mệnh đề. */
const BOOL_KEYWORDS = new Set(["AND", "OR"]);

/** Từ khoá cần khoảng trắng trước dấu `(` (phân biệt với lời gọi hàm). */
const KEYWORDS_BEFORE_PAREN = new Set([
  "IN",
  "EXISTS",
  "VALUES",
  "ALL",
  "ANY",
  "AND",
  "OR",
  "ON",
  "NOT",
  "BETWEEN",
]);

/** Danh sách từ khoá được viết hoa khi format. */
const UPPERCASE_WORDS = new Set([
  ...CLAUSE_KEYWORDS,
  ...JOIN_KEYWORDS,
  ...BOOL_KEYWORDS,
  "AS",
  "ON",
  "IN",
  "IS",
  "NOT",
  "NULL",
  "LIKE",
  "ILIKE",
  "BETWEEN",
  "EXISTS",
  "ALL",
  "ANY",
  "DISTINCT",
  "BY",
  "ASC",
  "DESC",
  "INTO",
  "OUTER",
  "INNER",
  "LEFT",
  "RIGHT",
  "FULL",
  "CROSS",
  "JOIN",
  "GROUP",
  "ORDER",
  "CASE",
  "WHEN",
  "THEN",
  "ELSE",
  "END",
  "COUNT",
  "SUM",
  "AVG",
  "MIN",
  "MAX",
  "TRUE",
  "FALSE",
  "AND",
  "OR",
]);

type Token = { type: "string" | "comment" | "word" | "punct" | "op"; value: string };

/** Tách câu lệnh thành token, giữ nguyên chuỗi và comment. */
function tokenize(sql: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  const n = sql.length;

  while (i < n) {
    const ch = sql[i];

    // Bỏ khoảng trắng.
    if (/\s/.test(ch)) {
      i += 1;
      continue;
    }

    // Comment dòng: -- ... đến hết dòng.
    if (ch === "-" && sql[i + 1] === "-") {
      let j = i + 2;
      while (j < n && sql[j] !== "\n") j += 1;
      tokens.push({ type: "comment", value: sql.slice(i, j) });
      i = j;
      continue;
    }

    // Comment khối: /* ... */
    if (ch === "/" && sql[i + 1] === "*") {
      let j = i + 2;
      while (j < n && !(sql[j] === "*" && sql[j + 1] === "/")) j += 1;
      j = Math.min(n, j + 2);
      tokens.push({ type: "comment", value: sql.slice(i, j) });
      i = j;
      continue;
    }

    // Chuỗi 'literal' (hỗ trợ escape '' bên trong).
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
      tokens.push({ type: "string", value: sql.slice(i, j) });
      i = j;
      continue;
    }

    // Định danh có dấu ngoặc kép "id".
    if (ch === '"') {
      let j = i + 1;
      while (j < n && sql[j] !== '"') j += 1;
      j = Math.min(n, j + 1);
      tokens.push({ type: "string", value: sql.slice(i, j) });
      i = j;
      continue;
    }

    // Dấu câu đơn.
    if ("(),;".includes(ch)) {
      tokens.push({ type: "punct", value: ch });
      i += 1;
      continue;
    }

    // Từ / số / định danh.
    if (/[A-Za-z0-9_$.]/.test(ch)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_$.]/.test(sql[j])) j += 1;
      tokens.push({ type: "word", value: sql.slice(i, j) });
      i = j;
      continue;
    }

    // Toán tử / ký hiệu còn lại (gom các ký tự liền nhau).
    let j = i;
    while (j < n && !/[\sA-Za-z0-9_$.'"(),;]/.test(sql[j])) j += 1;
    tokens.push({ type: "op", value: sql.slice(i, j) });
    i = j;
  }

  return tokens;
}

function isWord(token: Token | undefined, value: string): boolean {
  return !!token && token.type === "word" && token.value.toUpperCase() === value;
}

/**
 * Format một câu lệnh SQL. Trả về chuỗi đã beautify.
 * Nếu input rỗng thì trả nguyên input.
 */
export function formatSql(input: string): string {
  const sql = input.trim();
  if (!sql) return input;

  const tokens = tokenize(sql);
  let out = "";
  let depth = 0; // độ sâu ngoặc "block" (subquery đã xuống dòng).
  const parenStack: boolean[] = []; // mỗi "(": true nếu là block (đã xuống dòng).

  const indentUnit = "  ";
  const pad = (extra = 0) => indentUnit.repeat(Math.max(0, depth + extra));

  const trimTrailing = () => {
    out = out.replace(/[ \t]+$/, "");
  };
  // `true` nếu con trỏ đang ở đầu một dòng (chỉ có khoảng trắng thụt lề).
  const atLineStart = () => /(?:^|\n)[ \t]*$/.test(out);
  // Xuống dòng idempotent: nếu đã ở đầu dòng thì chỉ chỉnh lại mức thụt lề,
  // tránh tạo dòng trống thừa khi nhiều quy tắc cùng yêu cầu xuống dòng.
  const newline = (extra = 0) => {
    if (!out) return;
    if (atLineStart()) {
      out = out.replace(/[ \t]*$/, pad(extra));
    } else {
      trimTrailing();
      out += "\n" + pad(extra);
    }
  };
  let lastWordUpper = "";
  const needsSpaceBefore = (value: string) => {
    if (!out) return false;
    const last = out[out.length - 1];
    if (last === "\n" || last === " " || last === "(") return false;
    if (value === "," || value === ";" || value === ")" || value === "(") return false;
    if (value === ".") return false;
    if (last === ".") return false;
    return true;
  };
  const append = (value: string, forceSpace = false) => {
    if (forceSpace || needsSpaceBefore(value)) out += " ";
    out += value;
  };

  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i];
    const upper = token.value.toUpperCase();

    if (token.type === "comment") {
      if (out && !atLineStart()) newline();
      out += token.value;
      newline();
      lastWordUpper = "";
      continue;
    }

    if (token.type === "string") {
      append(token.value);
      lastWordUpper = "";
      continue;
    }

    if (token.type === "punct") {
      if (token.value === "(") {
        // Kiểm tra subquery: "(" theo sau (bỏ qua) là SELECT.
        const next = tokens[i + 1];
        const isBlock = isWord(next, "SELECT");
        append("(", KEYWORDS_BEFORE_PAREN.has(lastWordUpper));
        parenStack.push(isBlock);
        if (isBlock) {
          depth += 1;
          newline();
        }
        lastWordUpper = "";
        continue;
      }
      if (token.value === ")") {
        const wasBlock = parenStack.pop() ?? false;
        if (wasBlock) {
          depth = Math.max(0, depth - 1);
          newline();
        }
        out += ")";
        lastWordUpper = "";
        continue;
      }
      if (token.value === ",") {
        trimTrailing();
        out += ",";
        lastWordUpper = "";
        // Chỉ xuống dòng cho danh sách ở mức ngoài cùng (SELECT / GROUP BY / ORDER BY),
        // không áp dụng bên trong ngoặc (đối số hàm, danh sách IN, ...).
        if (parenStack.length === 0) {
          newline(1);
        }
        continue;
      }
      if (token.value === ";") {
        trimTrailing();
        out += ";";
        newline();
        lastWordUpper = "";
        continue;
      }
      continue;
    }

    if (token.type === "op") {
      append(token.value);
      lastWordUpper = "";
      continue;
    }

    // token.type === "word"
    const display = UPPERCASE_WORDS.has(upper) ? upper : token.value;

    // Cụm 2/3 từ (GROUP BY, LEFT OUTER JOIN, ...).
    const next = tokens[i + 1];
    const next2 = tokens[i + 2];
    const twoWord = next && next.type === "word" ? `${upper} ${next.value.toUpperCase()}` : "";
    const threeWord =
      next && next2 && next.type === "word" && next2.type === "word"
        ? `${upper} ${next.value.toUpperCase()} ${next2.value.toUpperCase()}`
        : "";

    if (threeWord && CLAUSE_PHRASES.has(threeWord)) {
      newline();
      out += `${upper} ${next.value.toUpperCase()} ${next2.value.toUpperCase()}`;
      lastWordUpper = next2.value.toUpperCase();
      i += 2;
      continue;
    }
    if (twoWord && CLAUSE_PHRASES.has(twoWord)) {
      newline();
      out += `${upper} ${next.value.toUpperCase()}`;
      lastWordUpper = next.value.toUpperCase();
      i += 1;
      continue;
    }

    if (CLAUSE_KEYWORDS.has(upper)) {
      newline();
      out += display;
      lastWordUpper = upper;
      continue;
    }
    if (JOIN_KEYWORDS.has(upper)) {
      newline();
      out += display;
      lastWordUpper = upper;
      continue;
    }
    if (BOOL_KEYWORDS.has(upper)) {
      newline(1);
      out += display;
      lastWordUpper = upper;
      continue;
    }

    append(display);
    lastWordUpper = upper;
  }

  return out.replace(/[ \t]+$/gm, "").replace(/\n{3,}/g, "\n\n").trim();
}

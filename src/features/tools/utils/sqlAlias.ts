/**
 * Phân tích các mệnh đề FROM / JOIN để lập bản đồ alias → tên bảng.
 *
 * Dùng cho autocomplete: khi người dùng gõ `p.` thì biết `p` trỏ tới bảng nào để
 * chỉ gợi ý đúng cột của bảng đó. Đây là parser "đủ dùng" (không phải parser SQL
 * đầy đủ) — bỏ qua chuỗi/comment, đọc các tham chiếu bảng sau FROM/JOIN.
 */

/** Từ khoá kết thúc một danh sách tham chiếu bảng. */
const STOP_WORDS = new Set([
  "where", "group", "order", "having", "limit", "offset", "on", "using",
  "and", "or", "join", "inner", "left", "right", "full", "cross", "outer",
  "union", "except", "intersect", "select", "set", "values", "returning",
  "into", "as", "from",
]);

/** Bỏ chuỗi literal và comment (thay bằng khoảng trắng) để không nhiễu khi tách token. */
function stripNoise(sql: string): string {
  return sql
    .replace(/--[^\n]*/g, " ")
    .replace(/\/\*[\s\S]*?\*\//g, " ")
    .replace(/'(?:''|[^'])*'/g, " ");
}

/** Tách thành token: định danh (kể cả "quoted"), và các ký tự `. , ( ) ;`. */
function tokenize(sql: string): string[] {
  const tokens: string[] = [];
  const re = /"[^"]*"|[A-Za-z_][A-Za-z0-9_$]*|\.|,|\(|\)|;/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(sql)) !== null) tokens.push(match[0]);
  return tokens;
}

function unquote(value: string): string {
  return value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value;
}

function isName(token: string | undefined): boolean {
  return !!token && /^(?:"[^"]*"|[A-Za-z_][A-Za-z0-9_$]*)$/.test(token);
}

/**
 * Trả về Map (khoá viết thường) gồm cả alias → tên bảng và tên bảng → tên bảng.
 */
export function parseTableAliases(sql: string): Map<string, string> {
  const tokens = tokenize(stripNoise(sql));
  const map = new Map<string, string>();
  let i = 0;

  const register = (key: string, table: string) => {
    const clean = unquote(key);
    if (clean) map.set(clean.toLowerCase(), table);
  };

  while (i < tokens.length) {
    const word = tokens[i].toLowerCase();
    if (word !== "from" && word !== "join") {
      i += 1;
      continue;
    }
    i += 1; // bỏ qua FROM/JOIN

    // Đọc lần lượt các tham chiếu bảng (FROM có thể có nhiều, ngăn bởi dấu phẩy).
    // eslint-disable-next-line no-constant-condition
    while (true) {
      // Subquery: FROM (SELECT ...) alias → bỏ qua phần trong ngoặc.
      if (tokens[i] === "(") {
        let depth = 0;
        while (i < tokens.length) {
          if (tokens[i] === "(") depth += 1;
          else if (tokens[i] === ")") {
            depth -= 1;
            if (depth === 0) {
              i += 1;
              break;
            }
          }
          i += 1;
        }
        // alias sau subquery (nếu có) → không map được cột thật, bỏ qua.
        if (isName(tokens[i]) && !STOP_WORDS.has(tokens[i].toLowerCase())) i += 1;
      } else if (isName(tokens[i])) {
        // Tên bảng, có thể có tiền tố schema: a.b.c → lấy phần cuối làm tên bảng.
        let table = unquote(tokens[i]);
        i += 1;
        while (tokens[i] === "." && isName(tokens[i + 1])) {
          table = unquote(tokens[i + 1]);
          i += 2;
        }

        // Alias: `AS name` hoặc `name` (nếu không phải từ khoá dừng).
        let alias = table;
        if (tokens[i] && tokens[i].toLowerCase() === "as" && isName(tokens[i + 1])) {
          alias = unquote(tokens[i + 1]);
          i += 2;
        } else if (isName(tokens[i]) && !STOP_WORDS.has(tokens[i].toLowerCase())) {
          alias = unquote(tokens[i]);
          i += 1;
        }

        register(table, table);
        register(alias, table);
      } else {
        break;
      }

      // Còn dấu phẩy → tiếp tục tham chiếu tiếp theo; ngược lại dừng.
      if (tokens[i] === ",") {
        i += 1;
        continue;
      }
      break;
    }
  }

  return map;
}

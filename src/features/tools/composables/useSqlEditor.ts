import { computed, reactive, ref, watch } from "vue";
import {
  sqlDeleteConnection,
  sqlGetSchema,
  sqlListConnections,
  sqlRunQuery,
  sqlSaveConnection,
  sqlTestConnection,
} from "@/tauri/commands/sql-editor";
import { friendlyError } from "@/tauri/commands/_base";
import { formatSql } from "../utils/formatSql";
import { parseTableAliases } from "../utils/sqlAlias";
import {
  DB_TYPE_OPTIONS,
  type DbTable,
  type QueryResult,
  type SaveSqlConnectionRequest,
  type SqlConnection,
} from "@/_/types/sql-editor";

/** Một mục gợi ý autocomplete. */
export type SuggestItem = {
  label: string;
  kind: "keyword" | "table" | "column";
  detail?: string;
};

/** Từ khoá SQL phổ biến dùng cho autocomplete. */
const SQL_KEYWORDS = [
  "SELECT", "FROM", "WHERE", "GROUP BY", "ORDER BY", "HAVING", "LIMIT", "OFFSET",
  "JOIN", "LEFT JOIN", "RIGHT JOIN", "INNER JOIN", "FULL JOIN", "ON", "AS",
  "AND", "OR", "NOT", "IN", "IS", "NULL", "LIKE", "ILIKE", "BETWEEN", "EXISTS",
  "INSERT INTO", "VALUES", "UPDATE", "SET", "DELETE FROM", "RETURNING",
  "DISTINCT", "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "CASE", "WHEN",
  "THEN", "ELSE", "END", "UNION", "UNION ALL", "ASC", "DESC", "TRUE", "FALSE",
];

/** Một tab query độc lập ở cột bên phải (có query text + kết quả riêng). */
export type QueryTab = {
  id: number;
  title: string;
  query: string;
  result: QueryResult | null;
  running: boolean;
  error: string;
};

/** Giá trị mặc định cho form thêm mới một kết nối. */
function emptyForm(): SaveSqlConnectionRequest {
  return {
    id: 0,
    name: "",
    db_type: "postgres",
    host: "localhost",
    port: 5432,
    database: "",
    username: "postgres",
    password: "",
  };
}

// ── Lưu phiên làm việc (tabs + query đã viết) để giữ khi rời màn hình / khởi động lại ──
const STORAGE_KEY = "sqlEditor.session";

type PersistedTab = { id: number; title: string; query: string };
type PersistedSession = {
  tabs: PersistedTab[];
  activeTabId: number;
  activeId: number | null;
};

function loadSession(): PersistedSession | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const data = JSON.parse(raw) as PersistedSession;
    if (!data || !Array.isArray(data.tabs) || data.tabs.length === 0) return null;
    return data;
  } catch {
    return null;
  }
}

function createSqlEditor() {
  const connections = ref<SqlConnection[]>([]);
  const activeId = ref<number | null>(null);

  // Cache schema (bảng + cột) theo id kết nối để phục vụ autocomplete.
  const schemaByConn = ref<Record<number, DbTable[]>>({});
  const schema = computed<DbTable[]>(() =>
    activeId.value !== null ? schemaByConn.value[activeId.value] ?? [] : [],
  );

  const form = reactive<SaveSqlConnectionRequest>(emptyForm());
  const showForm = ref(false);
  const isNew = computed(() => !form.id || form.id === 0);

  // ── Query tabs ─────────────────────────────────────────────────────────
  let tabSeq = 0;
  function makeTab(query = "SELECT * FROM projects LIMIT 100;"): QueryTab {
    tabSeq += 1;
    return {
      id: tabSeq,
      title: `Query ${tabSeq}`,
      query,
      result: null,
      running: false,
      error: "",
    };
  }

  const tabs = ref<QueryTab[]>([makeTab()]);
  const activeTabId = ref<number>(tabs.value[0].id);
  const activeTab = computed(
    () => tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  );

  // Khôi phục phiên trước (nếu có): chỉ giữ nội dung query, không giữ kết quả.
  const saved = loadSession();
  if (saved) {
    tabs.value = saved.tabs.map((t) => ({
      id: t.id,
      title: t.title,
      query: t.query,
      result: null,
      running: false,
      error: "",
    }));
    tabSeq = Math.max(0, ...saved.tabs.map((t) => t.id));
    activeTabId.value = saved.tabs.some((t) => t.id === saved.activeTabId)
      ? saved.activeTabId
      : tabs.value[0].id;
    if (typeof saved.activeId === "number") activeId.value = saved.activeId;
  }

  // Lưu lại phiên mỗi khi tab / query / kết nối đang chọn thay đổi.
  // Getter chỉ chạm vào id/title/query nên reactivity không phải duyệt sâu `result`.
  watch(
    () =>
      JSON.stringify({
        tabs: tabs.value.map((t) => ({ id: t.id, title: t.title, query: t.query })),
        activeTabId: activeTabId.value,
        activeId: activeId.value,
      } satisfies PersistedSession),
    (json) => {
      try {
        localStorage.setItem(STORAGE_KEY, json);
      } catch {
        // Bỏ qua lỗi lưu (ví dụ storage đầy) — không ảnh hưởng thao tác.
      }
    },
  );

  const loading = ref(false);
  const saving = ref(false);
  const testing = ref(false);
  const error = ref("");
  const notice = ref("");

  const activeConnection = computed(
    () => connections.value.find((c) => c.id === activeId.value) ?? null,
  );

  function clearMessages() {
    error.value = "";
    notice.value = "";
  }

  function newTab() {
    const tab = makeTab("");
    tabs.value.push(tab);
    activeTabId.value = tab.id;
  }

  function selectTab(id: number) {
    activeTabId.value = id;
  }

  function closeTab(id: number) {
    const index = tabs.value.findIndex((t) => t.id === id);
    if (index === -1) return;
    tabs.value.splice(index, 1);
    // Luôn giữ ít nhất một tab.
    if (tabs.value.length === 0) {
      const tab = makeTab();
      tabs.value.push(tab);
      activeTabId.value = tab.id;
      return;
    }
    // Nếu đóng đúng tab đang mở → chuyển sang tab liền kề.
    if (activeTabId.value === id) {
      const next = tabs.value[Math.max(0, index - 1)];
      activeTabId.value = next.id;
    }
  }

  async function loadConnections() {
    loading.value = true;
    clearMessages();
    try {
      connections.value = await sqlListConnections();
      // Bỏ chọn nếu kết nối đang active (khôi phục từ phiên trước) không còn tồn tại.
      if (activeId.value !== null && !connections.value.some((c) => c.id === activeId.value)) {
        activeId.value = null;
      }
      // Tự chọn kết nối đầu tiên nếu chưa có kết nối nào đang active.
      if (activeId.value === null && connections.value.length > 0) {
        activeId.value = connections.value[0].id;
      }
      void loadSchema();
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      loading.value = false;
    }
  }

  function selectConnection(id: number) {
    activeId.value = id;
    clearMessages();
    void loadSchema();
  }

  /** Nạp schema cho kết nối đang active (dùng cache; bỏ qua lỗi vì không bắt buộc). */
  async function loadSchema(force = false) {
    const id = activeId.value;
    if (id === null) return;
    if (!force && schemaByConn.value[id]) return;
    try {
      schemaByConn.value = { ...schemaByConn.value, [id]: await sqlGetSchema(id) };
    } catch {
      // Autocomplete là tính năng phụ — thất bại thì im lặng.
    }
  }

  /** Gom cột (bỏ trùng theo tên) từ một danh sách bảng thành các mục gợi ý. */
  function columnsOf(tables: DbTable[]): SuggestItem[] {
    const items: SuggestItem[] = [];
    const seen = new Set<string>();
    for (const table of tables) {
      for (const column of table.columns) {
        const key = column.toLowerCase();
        if (seen.has(key)) continue;
        seen.add(key);
        items.push({ label: column, kind: "column", detail: table.name });
      }
    }
    return items;
  }

  /**
   * Lọc danh sách gợi ý theo từ đang gõ.
   * @param word Từ đang gõ (có thể rỗng).
   * @param qualifier Tiền tố trước dấu `.` (ví dụ `p` trong `p.col`), hoặc `null`.
   * @param sql Toàn bộ nội dung query — dùng để phân giải alias trong FROM/JOIN.
   */
  function getSuggestions(
    word: string,
    qualifier: string | null,
    sql: string,
  ): SuggestItem[] {
    const needle = word.toLowerCase();

    // Sau `alias.` → chỉ gợi ý cột của đúng bảng mà alias trỏ tới.
    if (qualifier !== null) {
      const table = parseTableAliases(sql).get(qualifier.toLowerCase());
      let tables = schema.value;
      if (table) {
        const matched = schema.value.filter(
          (t) => t.name.toLowerCase() === table.toLowerCase(),
        );
        // Chỉ thu hẹp khi tìm được bảng trong schema; nếu không (subquery, typo)
        // thì fallback về tất cả cột cho tiện.
        if (matched.length > 0) tables = matched;
      }
      return columnsOf(tables)
        .filter((i) => i.label.toLowerCase().startsWith(needle))
        .slice(0, 50);
    }

    // Chế độ thường: cột + bảng + từ khoá.
    const items: SuggestItem[] = columnsOf(schema.value);
    for (const table of schema.value) {
      items.push({ label: table.name, kind: "table", detail: table.schema });
    }
    for (const keyword of SQL_KEYWORDS) {
      items.push({ label: keyword, kind: "keyword" });
    }

    const filtered = needle
      ? items.filter((i) => i.label.toLowerCase().startsWith(needle))
      : items;
    return filtered.slice(0, 50);
  }

  function openNewForm() {
    Object.assign(form, emptyForm());
    showForm.value = true;
    clearMessages();
  }

  function openEditForm(connection: SqlConnection) {
    Object.assign(form, {
      id: connection.id,
      name: connection.name,
      db_type: connection.db_type,
      host: connection.host,
      port: connection.port,
      database: connection.database,
      username: connection.username,
      password: connection.password,
    });
    showForm.value = true;
    clearMessages();
  }

  function closeForm() {
    showForm.value = false;
    clearMessages();
  }

  /** Đổi loại DB → gợi ý port mặc định nếu người dùng chưa đổi port thủ công. */
  function onDbTypeChange(value: string) {
    const option = DB_TYPE_OPTIONS.find((o) => o.value === value);
    form.db_type = value;
    if (option && option.defaultPort > 0) {
      form.port = option.defaultPort;
    }
  }

  async function saveConnection() {
    saving.value = true;
    clearMessages();
    try {
      const saved = await sqlSaveConnection({ ...form });
      await loadConnections();
      activeId.value = saved.id;
      showForm.value = false;
      notice.value = "Đã lưu kết nối.";
      void loadSchema(true); // nạp lại schema cho kết nối vừa lưu.
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      saving.value = false;
    }
  }

  async function testConnection() {
    testing.value = true;
    clearMessages();
    try {
      await sqlTestConnection({ ...form });
      notice.value = "Kết nối thành công.";
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      testing.value = false;
    }
  }

  async function removeConnection(id: number) {
    clearMessages();
    try {
      await sqlDeleteConnection(id);
      if (activeId.value === id) {
        activeId.value = null;
      }
      if (form.id === id) {
        showForm.value = false;
      }
      await loadConnections();
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  /**
   * Chạy query trên tab đang mở.
   * @param overrideSql Nếu truyền vào, chạy câu lệnh này thay cho nội dung tab
   *   (dùng cho EXPLAIN / EXPLAIN ANALYZE) mà không sửa nội dung editor.
   */
  async function runQuery(overrideSql?: string) {
    const tab = activeTab.value;
    if (!tab) return;
    if (activeId.value === null) {
      tab.error = "Vui lòng chọn một kết nối trước khi chạy query.";
      return;
    }
    const statement = (overrideSql ?? tab.query).trim();
    if (!statement) {
      tab.error = "Câu lệnh SQL đang trống.";
      return;
    }
    tab.running = true;
    tab.error = "";
    notice.value = "";
    try {
      tab.result = await sqlRunQuery(activeId.value, statement);
    } catch (e) {
      tab.result = null;
      tab.error = friendlyError(e);
    } finally {
      tab.running = false;
    }
  }

  /** Chạy EXPLAIN (hoặc EXPLAIN ANALYZE) cho nội dung tab đang mở. */
  function explain(analyze = false) {
    const tab = activeTab.value;
    if (!tab || !tab.query.trim()) return;
    const prefix = analyze ? "EXPLAIN ANALYZE " : "EXPLAIN ";
    return runQuery(prefix + tab.query.trim());
  }

  /** Beautify nội dung tab đang mở. */
  function formatActiveQuery() {
    const tab = activeTab.value;
    if (!tab) return;
    tab.query = formatSql(tab.query);
  }

  /** Xoá trắng nội dung tab đang mở. */
  function clearQuery() {
    const tab = activeTab.value;
    if (tab) tab.query = "";
  }

  /** Copy nội dung query vào clipboard. */
  async function copyQuery() {
    const tab = activeTab.value;
    if (!tab || !tab.query) return;
    try {
      await navigator.clipboard.writeText(tab.query);
      notice.value = "Đã copy query.";
    } catch {
      tab.error = "Không copy được vào clipboard.";
    }
  }

  /** Xuất kết quả của tab đang mở ra file CSV. */
  function exportCsv() {
    const tab = activeTab.value;
    const result = tab?.result;
    if (!result || !result.has_result_set) return;

    const escape = (cell: string | null) => {
      const value = cell ?? "";
      return /[",\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
    };
    const header = result.columns.map(escape).join(",");
    const body = result.rows.map((row) => row.map(escape).join(",")).join("\n");
    const csv = `${header}\n${body}`;

    const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${(tab?.title ?? "query").replace(/\s+/g, "_")}_result.csv`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  return {
    connections,
    activeId,
    activeConnection,
    schema,
    loadSchema,
    getSuggestions,
    form,
    showForm,
    isNew,
    tabs,
    activeTabId,
    activeTab,
    loading,
    saving,
    testing,
    error,
    notice,
    dbTypeOptions: DB_TYPE_OPTIONS,
    loadConnections,
    selectConnection,
    openNewForm,
    openEditForm,
    closeForm,
    onDbTypeChange,
    saveConnection,
    testConnection,
    removeConnection,
    newTab,
    selectTab,
    closeTab,
    runQuery,
    explain,
    formatActiveQuery,
    clearQuery,
    copyQuery,
    exportCsv,
  };
}

// State dùng chung ở phạm vi module (singleton): sống suốt vòng đời app nên khi
// điều hướng rời màn SQL Editor rồi quay lại, các tab + query vẫn được giữ nguyên.
let singleton: ReturnType<typeof createSqlEditor> | null = null;

export function useSqlEditor() {
  if (!singleton) singleton = createSqlEditor();
  return singleton;
}

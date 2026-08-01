# Đề xuất chuẩn hoá giao diện (Style Standardization)

Mục tiêu: gom các cách viết khác nhau về **cỡ chữ, font, màu chữ, màu nền, tiêu đề, banner, badge, focus ring** về một bộ chuẩn duy nhất, bám đúng cơ chế hiện có:

`themeTokens.ts` (CSS variables) → `tailwind.config.js` (map thành class) → `styles.css` (`@layer components` cho class dùng lại).

> Nguyên tắc vàng: **Không dùng màu palette thô (`slate-*`, `surface-*`, `red-*`, `emerald-*`…) và không dùng `text-[..px]` tuỳ ý.** Mọi thứ đi qua token / class chuẩn.

---

## 0. Một nguồn sự thật (SINGLE SOURCE OF TRUTH) — quan trọng nhất

**Chỉ khai báo giá trị màu ở duy nhất `src/shared/config/themeTokens.ts`.** Mọi nơi khác chỉ được *tham chiếu* qua `rgb(var(--color-*))`, không bao giờ chép lại giá trị.

### Hiện trạng: giá trị đang bị lặp ở 3 nơi (cần gom lại trước khi thêm token mới)

| Nơi | Nội dung | Xử lý |
|-----|----------|-------|
| `themeTokens.ts` `LIGHT_THEME`/`DARK_THEME` | nguồn chính | ✅ GIỮ làm nguồn duy nhất |
| `styles.css` `.force-light` (dòng ~15–28) | chép lại `250 250 249`, `13 147 115`… | ❌ BỎ literal, sinh tự động từ `LIGHT_THEME` |
| `styles.css` override PrimeVue (dòng ~30–76) | hex `#0d9373`/`#0f172a`/`#e5e7eb`/`#ffffff` | ❌ đổi sang `rgb(var(--color-brand))`… |

Đối chiếu: `#0d9373`=brand, `#0f172a`=ink, `#e5e7eb`=border, `#ffffff`=panel → tất cả đều đã có token, thay được 100%.

### Cách bỏ trùng lặp cho `.force-light`

`.force-light` cần literal vì nó ép giá trị **light** đè lên `:root` (kể cả khi root đang dark) cho màn Login. Thay vì viết tay trong CSS, **sinh từ `LIGHT_THEME`** — vẫn 1 nguồn:

```ts
// themeTokens.ts — chạy 1 lần lúc khởi động (main.ts)
export function injectForceLightStyle() {
  const decls = [
    ...Object.entries(LIGHT_THEME.colors).map(([k, v]) => `--color-${k}: ${v};`),
    ...Object.entries(LIGHT_THEME.shadows).map(([k, v]) => `--shadow-${k}: ${v};`),
  ].join("");
  const style = document.createElement("style");
  style.textContent = `.force-light{${decls}}`;
  document.head.appendChild(style);
}
```

Rồi trong `styles.css`, block `.force-light` chỉ còn phần *không phải giá trị màu* (nếu có), và các override PrimeVue đổi thành:

```css
.force-light .p-inputtext { background: rgb(var(--color-panel)); color: rgb(var(--color-ink)); border-color: rgb(var(--color-border)); }
.force-light .p-button    { background: rgb(var(--color-brand)); border-color: rgb(var(--color-brand)); }
/* … không còn hex nào */
```

→ Sau này đổi bảng màu **chỉ sửa `themeTokens.ts`**, `.force-light` + PrimeVue + Tailwind tự cập nhật theo.

### Hệ quả cho các token mới ở mục 2

Token state (`danger/warning/success/info`, `on-brand`, `code`) **chỉ thêm vào `LIGHT_THEME`/`DARK_THEME`** trong `themeTokens.ts`. Vì `.force-light` được sinh từ `LIGHT_THEME`, nó tự có luôn — không phải khai báo lại ở đâu.

---

## 1. Thang cỡ chữ (font-size scale)

Chỉ dùng đúng các mức sau; **cấm** `text-[8px]`, `text-[9px]`, `text-[10px]`, `text-[11px]`, `text-[13px]`, `text-[15px]`.

| Class | px | Dùng cho |
|-------|-----|----------|
| `text-2xs` *(thêm mới)* | 11px | badge nhỏ, meta dày đặc (thay cho `text-[10px]`/`text-[11px]`) |
| `text-xs` | 12px | nhãn field, phụ chú, count |
| `text-sm` | 14px | nội dung phụ, mô tả, nội dung bảng |
| `text-base` | 15px | nội dung chính (mặc định) |
| `text-lg` | 18px | **tiêu đề trang** |
| `text-xl` | 20px | tiêu đề màn login/dialog lớn |
| `text-2xl` | 24px | số liệu lớn (summary card) |

Thêm `text-2xs` vào `tailwind.config.js`:

```js
// theme.extend
fontSize: {
  "2xs": ["0.6875rem", { lineHeight: "1rem" }], // 11px
},
```

**Quy đổi:** `text-[11px]`→`text-2xs`, `text-[10px]`/`text-[9px]`/`text-[8px]`→`text-2xs`, `text-[13px]`→`text-xs`, `text-[15px]`→`text-base`.

---

## 2. Token màu trạng thái (state colors)

Hiện mỗi trang tự chọn sắc độ đỏ/vàng/xanh khác nhau. Thêm 4 nhóm token: **danger / warning / success / info**, mỗi nhóm có `fg` (chữ), `soft` (nền nhạt), `border`.

### 2a. `themeTokens.ts` — thêm vào `colors` của cả LIGHT và DARK

```
LIGHT:
  "danger":         "185 28 28",    "danger-soft":  "254 242 242",  "danger-border":  "254 202 202",
  "warning":        "146 64 14",    "warning-soft": "255 251 235",  "warning-border": "253 230 138",
  "success":        "4 120 87",     "success-soft": "236 253 245",  "success-border": "167 243 208",
  "info":           "29 78 216",    "info-soft":    "239 246 255",  "info-border":    "191 219 254",
  "on-brand":       "255 255 255",   // chữ trên nền brand (thay text-white)

DARK:
  "danger":         "252 165 165",  "danger-soft":  "69 10 10",     "danger-border":  "153 27 27",
  "warning":        "252 211 77",   "warning-soft": "69 26 3",      "warning-border": "146 64 14",
  "success":        "134 239 172",  "success-soft": "6 78 59",      "success-border": "6 95 70",
  "info":           "147 197 253",  "info-soft":    "23 37 84",     "info-border":    "30 64 175",
  "on-brand":       "255 255 255",
```

### 2b. `tailwind.config.js` — map thành class

```js
danger:  "rgb(var(--color-danger) / <alpha-value>)",
warning: "rgb(var(--color-warning) / <alpha-value>)",
success: "rgb(var(--color-success) / <alpha-value>)",
info:    "rgb(var(--color-info) / <alpha-value>)",
"danger-soft":   "rgb(var(--color-danger-soft) / <alpha-value>)",
"danger-border": "rgb(var(--color-danger-border) / <alpha-value>)",
// … tương tự warning/success/info
"on-brand": "rgb(var(--color-on-brand) / <alpha-value>)",
```

→ Dùng: `text-danger`, `bg-danger-soft`, `border-danger-border`… tự động đúng cả light/dark, bỏ hết `dark:` variant thủ công.

---

## 3. Class dùng lại (`@layer components` trong `styles.css`)

### 3a. Tiêu đề & header — thống nhất 1 chuẩn

```css
.page-title    { @apply text-lg font-semibold text-ink; }      /* tiêu đề trang */
.page-subtitle { @apply mt-1 text-sm text-muted; }             /* phụ đề dưới tiêu đề */
.section-title { @apply text-sm font-bold text-ink; }          /* tiêu đề card/section */
.section-eyebrow { @apply text-xs font-bold uppercase tracking-wide text-muted; } /* nhãn nhóm cột */
```

**Quy ước tag:** tiêu đề trang = `<h2 class="page-title">`, tiêu đề section = `<h3 class="section-title">`. Bỏ các kiểu lẫn lộn `h3 font-bold` (không cỡ), `h4 font-bold`, `h3 text-sm font-bold`.

### 3b. Banner thông báo — 4 loại, thay toàn bộ banner hardcode

```css
.banner        { @apply flex items-start gap-2 rounded-lg border px-3 py-2 text-sm; }
.banner-danger  { @apply banner border-danger-border  bg-danger-soft  text-danger; }
.banner-warning { @apply banner border-warning-border bg-warning-soft text-warning; }
.banner-success { @apply banner border-success-border bg-success-soft text-success; }
.banner-info    { @apply banner border-info-border    bg-info-soft    text-info; }
```

### 3c. Badge trạng thái — thay `bg-emerald-100/text-emerald-700`… rải rác

```css
.badge         { @apply inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-2xs font-bold; }
.badge-success { @apply badge bg-success-soft text-success; }
.badge-warning { @apply badge bg-warning-soft text-warning; }
.badge-danger  { @apply badge bg-danger-soft  text-danger; }
.badge-info    { @apply badge bg-info-soft    text-info; }
.badge-neutral { @apply badge bg-canvas       text-secondary; }
```

### 3d. Focus ring — chọn 1 chuẩn duy nhất

Bỏ `ring-emerald-100`, dùng thống nhất theo Login/Forgot:

```css
.field-ring { @apply focus:border-brand focus:ring-2 focus:ring-brand/20; }
/* biến thể focus-within cho wrapper input: focus-within:border-brand focus-within:ring-2 focus-within:ring-brand/20 */
```

---

## 4. Nền trang & thẻ (surface)

- **Root của trang:** KHÔNG đặt `bg-*` → thừa hưởng `bg-canvas`. (Bỏ `bg-panel` ở root của `ProjectDetailPage`, `DailyReportPage`.)
- **Card/section:** `rounded-lg border border-divider bg-panel shadow-sm` (đã là mẫu phổ biến — giữ làm chuẩn).
- **Xoá bảng màu `surface-*`:** `S3DownloadPage`, `S3UploadPage`:
  - `bg-surface-0` / `dark:bg-surface-900` → `bg-panel`
  - `text-surface-500` → `text-muted`
  - `border-surface-200` / `dark:border-surface-700` → `border-divider`
  - `text-surface-*` (chữ) → `text-ink` / `text-secondary` tuỳ cấp.

---

## 5. Chữ trên nền brand & màu hex

- `text-white` (trên `bg-brand`) → `text-on-brand`. Ảnh hưởng: AiChat, AiUsage, DailyReport, IssueBacklog, Governance Permissions/Users, Login (`!text-white`→`!text-on-brand`).
- Hex thẳng:
  - `bg-[#4cbd9b]` (DailyReport) → `bg-brand` (hoặc token riêng nếu cần sắc độ khác).
  - `background:#0b0f19` (Terminal), `bg-slate-950 text-slate-100` (CopyTools, Excel2md log/preview) → thêm cặp token **surface tối cố định** cho vùng "code/terminal": `--color-code-bg` / `--color-code-fg` (giống nhau ở cả 2 theme) rồi dùng `bg-code text-code-fg`.
  - Hex syntax-highlight SQL (SqlEditor, StoreProcedure) → gom về **một** bảng token dùng chung (vd `--sql-keyword`, `--sql-string`, `--sql-number`, `--sql-comment`) khai báo 1 lần, 2 file cùng tham chiếu (hiện đang copy hex ở cả hai).

---

## 6. Màu icon theo loại file — gom về 1 bảng dùng chung

Hiện folder lúc `text-amber-500` lúc `text-orange-500`, file icon mỗi trang một khác. Chuẩn hoá 1 bảng (đặt trong 1 util/constants và tái sử dụng ở ExploreFaster / S3 / FileSplit / GitDesktop):

| Loại | Class chuẩn |
|------|-------------|
| Folder | `text-amber-500` |
| Excel  | `text-green-600` |
| Word   | `text-blue-600` |
| PDF    | `text-red-600` |
| Image  | `text-purple-500` |
| Archive| `text-orange-500` |
| Code   | `text-cyan-500` |

(Đây là màu "thương hiệu file" — chấp nhận nằm ngoài token trạng thái, nhưng phải **thống nhất một chỗ**, không mỗi trang tự chế.)

---

## 7. Bảng quy đổi nhanh (migration cheat-sheet)

| Đang dùng (lệch) | Đổi thành (chuẩn) |
|---|---|
| `h3 font-bold` / `h4 font-bold` (tiêu đề trang) | `<h2 class="page-title">` |
| `h3 text-sm font-bold` (tiêu đề section) | `<h3 class="section-title">` |
| `text-[10px]` / `text-[11px]` / `[9px]` / `[8px]` | `text-2xs` |
| `text-[13px]` → `text-xs`, `text-[15px]` → `text-base` | thang chuẩn |
| `border-red-200 bg-red-50 text-red-800` (+dark) | `.banner-danger` |
| `border-amber-200 bg-amber-50 text-amber-800` | `.banner-warning` |
| `bg-green-500/10 text-green-700` / `bg-emerald-50 text-emerald-700` | `.banner-success` |
| `bg-blue-50 text-blue-800` | `.banner-info` |
| `bg-emerald-100 text-emerald-700` (badge) | `.badge-success` |
| `focus:ring-emerald-100` | `focus:ring-brand/20` (`.field-ring`) |
| `bg-surface-0` / `text-surface-500` / `border-surface-200` | `bg-panel` / `text-muted` / `border-divider` |
| `text-white` trên `bg-brand` | `text-on-brand` |
| `bg-[#4cbd9b]` | `bg-brand` |
| `#0b0f19` / `bg-slate-950` (terminal/log) | `bg-code text-code-fg` |

---

## 8. Thứ tự triển khai đề xuất

1. **Gom về một nguồn (mục 0) — làm trước tiên:** thêm `injectForceLightStyle()`, bỏ literal màu trong `.force-light`, đổi hex PrimeVue sang `var(--color-*)`. Không đổi giao diện, nhưng từ đây mọi màu chỉ còn 1 nơi.
2. **Nền tảng (không đổi giao diện):** thêm token state + `text-2xs` + `on-brand`/`code` vào `themeTokens.ts` & `tailwind.config.js`; thêm class `.page-title`/`.section-title`/`.banner-*`/`.badge-*`/`.field-ring` vào `styles.css`.
3. **Nhóm lệch nặng trước:** `S3DownloadPage`, `S3UploadPage` (bỏ `surface-*`), `DailyReportPage` (hex + `text-[..px]`).
4. **Tiêu đề & banner toàn app:** quét thay `h3 font-bold`→`.page-title`/`.section-title`, banner đỏ/vàng/xanh→`.banner-*`.
5. **Badge & focus ring:** thay `ring-emerald-100` và badge palette.
6. **Icon file & syntax token:** gom bảng dùng chung.

> Bước 1–2 an toàn tuyệt đối (chỉ gom nguồn + thêm mới, không đổi giao diện). Từ bước 3 nên làm theo từng nhóm màn hình để dễ review.

---

## 9. Trạng thái triển khai (đã hoàn thành)

- ✅ **Bước 1** — Gom nguồn: `injectForceLightStyle()`, bỏ literal `.force-light`, hex PrimeVue → `var(--color-*)`.
- ✅ **Bước 2** — Token state (`danger/warning/success/info`, `on-brand`, `code`) + `text-2xs` + class `.page-title`/`.section-title`/`.section-eyebrow`/`.banner-*`/`.badge-*`/`.field-ring`.
- ✅ **Bước 3** — Nhóm lệch nặng: S3Download/Upload (+card) bỏ `surface-*`; DailyReport bỏ hex `#4cbd9b` + `text-[..px]`.
- ✅ **Bước 4** — Tiêu đề & banner toàn app (~50 heading, ~18 banner).
- ✅ **Bước 5** — Focus ring `ring-brand/20` (12 file) + badge `.badge-*` (gồm tone/severity functions) toàn app.
- ✅ **Bước 5b** — Git dialogs: `fileStatus.ts` (`cls`+`badge`) + refClass/prStateBadge + inline badges → token.
- ✅ **Bước 6** — Folder icon `orange`→`amber` (S3BugFolders); SQL syntax hex → CSS var `--sql-*` (một nguồn ở `styles.css`, dùng chung SqlEditor + StoreProcedure).

### Ngoại lệ cố ý giữ nguyên
Tint dòng bảng/diff/lịch (`bg-*-50` theo `line.kind`/`isWeekend`); thanh usage/progress (`bg-red-500`); nút danger context-menu (`hover:bg-red-50`); icon archive (`text-orange-500` — đúng bảng chuẩn); `AppToast.vue` `#93c5fd` (component toast, ngoài phạm vi); đoạn code inline `bg-amber-100` trong AiUsage.

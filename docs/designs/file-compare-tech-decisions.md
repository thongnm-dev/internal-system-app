# File Compare Tool — Quyết định kỹ thuật

> Màn hình so sánh sự khác biệt giữa 2 file. Loại hỗ trợ: **Markdown, Excel, Word (.docx), Text**.
> Ghi lại ngày 2026-08-01 để tiếp tục triển khai sau này.

## 1. Kiến trúc tổng thể

Tool mới trong nhóm **Tools**:

- Frontend: `src/features/tools/components/FileComparePage.vue` + `composables/useFileCompare.ts`
- Backend: `src-tauri/src/commands/file_compare_commands.rs` (đăng ký trong `lib.rs`)
- Types: `src/_/types/file-compare.ts`
- IPC wrapper: thêm domain file trong `src/tauri/commands/` + re-export ở `index.ts`
- Route + menu: `src/app/router/routes.ts` (nhóm Tools)

**Luồng:** chọn 2 file (plugin-dialog) → Rust đọc & trích xuất về dạng chuẩn hóa → diff → trả frontend render.

**Không dùng stored procedure** — đây là xử lý file cục bộ, không đụng PostgreSQL.

## 2. Trích xuất theo loại file (Rust)

| Loại | Kỹ thuật | Chuẩn hóa |
|------|----------|-----------|
| Text / Markdown (`.txt`, `.md`) | `tokio::fs::read_to_string` | mảng dòng (lines) |
| Word (`.docx`) | crate `zip` mở `word/document.xml` → `roxmltree` duyệt `<w:p>` / `<w:t>` | mảng đoạn văn (paragraphs) |
| Excel (`.xlsx`, `.xls`) | crate `calamine` đọc từng sheet → row → col | ma trận cell |

- **Chỉ hỗ trợ `.docx`** (không hỗ trợ `.doc` binary cũ).
- Tất cả crate cần thiết (`zip`, `roxmltree`, `calamine`, `tokio`) **đã có sẵn** trong `Cargo.toml`.

## 3. Thuật toán diff

- **Chạy ở backend Rust** — thêm crate **`similar`** vào `Cargo.toml`.
- Text / Markdown / Word: `TextDiff` theo dòng → trả danh sách hunk
  `{ tag: equal | insert | delete, oldLine, newLine, content }`.
- Excel: **cell-by-cell** — union vùng ô của 2 file theo từng sheet, đánh dấu
  `changed | added | removed` kèm giá trị cũ/mới (không dùng diff dòng).

## 4. UI / Render (frontend)

- **Toggle chế độ hiển thị: Side-by-side (2 cột) ↔ Inline (1 cột)** — cho người dùng chọn.
- Text / MD / Word: tô màu insert (xanh) / delete (đỏ); cuộn đồng bộ ở chế độ 2 cột.
- Markdown: thêm toggle **raw diff / rendered** (dùng `marked` + `dompurify` đã có).
- Excel: bảng theo sheet (dropdown chọn sheet), tô nền ô khác biệt, tooltip giá trị cũ → mới.
- Form control dùng **PrimeVue** (`Select`, `SelectButton` cho toggle, `Button`) theo convention CLAUDE.md.

## 5. Ràng buộc & quyết định đã chốt

- 2 file **phải cùng loại**; khác loại → chặn và báo lỗi.
- Chọn file qua `@tauri-apps/plugin-dialog`.
- Diff engine: **Rust + `similar`**.
- Hiển thị: **có cả 2 chế độ 1 cột / 2 cột**, người dùng toggle.
- Word: **chỉ `.docx`**.
- Excel: **cell-by-cell**.

## 6. Thứ tự triển khai

1. ✅ Layout UI (`FileComparePage.vue`).
2. ✅ Route + seed menu (`fileCompare`, group Tools).
3. ✅ Backend Rust: `models/file_compare.rs`, `services/file_compare_service.rs`,
   `commands/file_compare_commands.rs`; crate `similar` đã thêm; đăng ký ở
   `modules.rs` + `lib.rs`. `cargo check` pass.
4. ✅ Types (`src/_/types/file-compare.ts`) + IPC wrapper
   (`src/tauri/commands/file-compare.ts`, re-export ở `index.ts`).
5. ✅ Composable `useFileCompare.ts` + nối `FileComparePage.vue` render diff thật
   (2 cột / inline / markdown rendered / bảng Excel). `vue-tsc` pass.
6. ✅ Tách phần kết quả ra `FileCompareResult.vue` (dùng lại inline + trong dialog);
   thêm nút phóng to mở `<Dialog maximizable>` full màn hình để tăng vùng hiển thị.
7. ✅ Xuất kết quả ra Excel (.xlsx): thêm crate `rust_xlsxwriter`; command
   `file_compare_export(fileA, fileB, outputPath)`; bảng báo cáo cột
   `STT | Sheet | Đối tượng | Thay đổi | Nội dung cũ | Nội dung mới`, tô màu theo
   loại (thêm/xóa/sửa). Excel: cột/dòng thêm-xóa gộp nội dung, ô sửa ghi old→new.
   Text/Word/MD: ghép delete+insert liền kề thành "Sửa". Nút "Xuất Excel" ở header
   kết quả (cả inline lẫn dialog) → hộp thoại `save` chọn nơi lưu.
8. ⏳ Chạy thử `npm run tauri:dev` với file thật (text/md/docx/xlsx) để kiểm thử end-to-end.

### Ghi chú kỹ thuật khi triển khai
- Excel đọc bằng `open_workbook_auto` (hỗ trợ cả `.xlsx/.xls/.xlsm`).
- `.docx`: mỗi `<w:p>` → 1 dòng; `<w:tab>`→`\t`, `<w:br>`→`\n`.
- Command IPC: `file_compare_run(fileA, fileB)` → `CompareResult { kind, textDiff, excelDiff }`.
- Side-by-side dựng lại 2 phía từ mảng `lines`: trái = tag≠insert, phải = tag≠delete.

### Excel: căn chỉnh cột + dòng (alignment) — tránh báo khác giả khi xóa cột/dòng
- **Không** so theo vị trí cố định nữa. Thuật toán 2 bước:
  1. **Căn cột** bằng LCS "mờ": mỗi cột có multiset giá trị (bỏ ô rỗng), 2 cột khớp khi
     độ tương đồng Sørensen–Dice ≥ 0.5 (`COL_SIM_THRESHOLD`). Chịu được cả thêm/xóa dòng.
  2. **Căn dòng** bằng Myers (`capture_diff_slices`) trên "khóa dòng" = ghép giá trị các
     **cột đã khớp** (sep `\u{1}`). `Replace` được ghép theo vị trí trong khối để ô sửa
     hiện thành `changed` thay vì xóa+thêm cả dòng.
- `SheetDiff` trả thêm `columns[]` / `rowsMeta[]` (`AxisMarker { tag, label }`) + đếm
  `colAdded/colRemoved/rowAdded/rowRemoved`. UI tô header cột/dòng bị thêm/xóa, cột phía
  sau cột-bị-xóa giữ nguyên (không còn báo `changed` giả).

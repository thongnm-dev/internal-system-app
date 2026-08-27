# TODO: PostgreSQL connection pooling (chưa làm)

## Vấn đề

`pgsql_connect::connect()` (`src-tauri/src/utils/pgsql_connect.rs:144`) mở một
kết nối TCP + auth handshake **mới tới Postgres ở mọi lần gọi** — không dùng
connection pool (`deadpool-postgres`, `bb8-postgres`, ...). Đây là pattern dùng
chung cho toàn bộ tầng `database/*_store.rs`.

- Tổng cộng **92 chỗ** gọi `pgsql_connect::connect()` trong `src-tauri/src/database/*.rs`.
- Riêng luồng S3 (được tối ưu ở phiên làm việc trước) có **14 chỗ** trong
  `aws_storage_store.rs`, `upload_store.rs`, `download_store.rs`.

Hệ quả: hầu hết thao tác S3 (upload_files, download_by_storage, move, delete,
list_bug_folder_tabs...) đều phải chờ 1-3 round-trip TCP+auth tới Postgres
**trước và/hoặc sau** khi gọi S3 (tra storage, tra work_folder trước khi cache,
ghi lịch sử upload/download...). Nếu DB ở xa (qua VPN/mạng công ty), mỗi
handshake có thể tốn hàng chục-hàng trăm ms — cộng dồn vào cảm giác "chậm" dù
phần S3 đã tối ưu.

## Việc cần làm khi triển khai

1. Thêm dependency pool, ví dụ `deadpool-postgres` (dùng chung `tokio-postgres`
   đang có sẵn, không cần đổi driver).
2. Tạo pool dùng chung (global, khởi tạo 1 lần khi app start hoặc lazy qua
   `OnceLock`/`OnceCell`), đọc cấu hình từ `config.ini` như hiện tại
   (`PgConfig::from_ini()`).
3. Đổi `pgsql_connect::connect()` thành lấy connection từ pool
   (`pool.get().await`) thay vì mở mới — giữ nguyên chữ ký hàm nếu có thể để
   tối thiểu hoá thay đổi ở 92 call site.
4. Cân nhắc: `connect_with(config)` (dùng cho SQL Editor, nơi người dùng tự
   nhập config tuỳ ý) — **không nên** đưa vào pool chung vì config có thể khác
   nhau mỗi lần; giữ nguyên tạo kết nối riêng cho trường hợp này.
5. Test kỹ vì đây là tầng dùng chung cho **toàn bộ ứng dụng** (không riêng
   S3) — cần chạy qua các luồng chính (auth, project, daily-report, issues,
   governance...) chứ không chỉ luồng S3.

## Phạm vi / rủi ro

Đây là thay đổi hạ tầng dùng chung, ảnh hưởng rộng hơn nhiều so với các tối ưu
S3 đã làm trước đó (vốn chỉ sửa `s3_service.rs`). Nên làm thành một phiên làm
việc riêng, review kỹ trước khi merge.

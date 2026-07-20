# System Manage Helper — Desktop App

## Stack

- **Frontend**: React + TypeScript + Vite, TailwindCSS, PrimeReact + PrimeIcons
- **Backend**: Tauri v2 (Rust) — `src-tauri/src/main.rs` chứa logic gom file

## Màn hình

## Lệnh

```bash
npm install            # cài dependency (chỉ lần đầu)
npm run tauri dev      # chạy app ở chế độ dev
npm run tauri build    # đóng gói bản cài đặt
```

## Cập nhật icon

```bash
npx tauri icon <đường-dẫn-png-nguồn>   # sinh lại bộ icon trong src-tauri/icons
```

## Thiết lập auto-update

App dùng `tauri-plugin-updater` (đã cài ở frontend `src/updater.ts` và backend
`src-tauri/Cargo.toml`) + workflow `.github/workflows/release.yml` (chạy khi push
tag `v*`, build cho macOS/Linux/Windows, ký artifact và sinh `latest.json` qua
`tauri-apps/tauri-action`). Cần làm 1 lần các bước sau để bật ký số:

1. **Sinh cặp khóa ký (chỉ làm 1 lần)** — chạy trong thư mục `app/`:

   ```bash
   npm run tauri signer generate -- -w ~/.tauri/significantly.key
   ```

   Lệnh sẽ hỏi một **password** để mã hóa private key (đây chính là giá trị
   dùng cho secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` ở bước 3) — cần nhớ
   lại password này. Kết quả sinh ra:
   - `~/.tauri/significantly.key` — **private key** (giữ bí mật, KHÔNG commit).
   - `~/.tauri/significantly.key.pub` — **public key** (an toàn để đưa vào repo).

2. **Dán public key vào config** — mở nội dung file `.pub` ở trên, thay vào
   placeholder `DÁN_PUBLIC_KEY_VÀO_ĐÂY` trong
   [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) (khóa
   `plugins.updater.pubkey`), rồi commit thay đổi này.

3. **Khai báo GitHub Actions secrets** — vào repo trên GitHub → Settings →
   Secrets and variables → Actions → **New repository secret**, thêm:
   - `TAURI_SIGNING_PRIVATE_KEY` — dán toàn bộ nội dung file `.key` (private
     key) ở bước 1.
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — password đã đặt ở bước 1.
   - `GITHUB_TOKEN` — **không cần tạo tay**, GitHub Actions tự cấp sẵn theo
     mỗi lần chạy workflow (đã tham chiếu qua `secrets.GITHUB_TOKEN` trong
     `release.yml`). Chỉ cần tạo secret này thủ công nếu muốn dùng một
     Personal Access Token riêng (ví dụ để publish sang repo khác).

4. **Phát hành bản cập nhật** — push tag đúng định dạng `v*` (vd `v1.0.0`):

   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

   Workflow `release.yml` sẽ tự build, ký từng artifact bằng
   `TAURI_SIGNING_PRIVATE_KEY`/`_PASSWORD`, và đính kèm `latest.json` vào
   GitHub Release (tạo ở chế độ draft — cần vào Release trên GitHub để publish
   thủ công). App cũ sẽ đọc `latest.json` này qua endpoint khai báo trong
   `plugins.updater.endpoints` để phát hiện và tải bản mới.

> Lưu ý: nếu tạo lại cặp khóa mới (key rotation), phải cập nhật lại cả
> `pubkey` trong `tauri.conf.json` lẫn 2 secret ở bước 3 — nếu không app cũ sẽ
> không xác thực được artifact mới và auto-update sẽ báo lỗi chữ ký.

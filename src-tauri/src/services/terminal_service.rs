//! Service quản lý các phiên terminal nhúng (pseudo-terminal / PTY).
//!
//! Mỗi phiên là một tiến trình shell thật chạy trong một PTY do `portable-pty`
//! cấp phát. Service giữ một registry `id -> Session` để frontend có thể:
//! - `spawn` — mở phiên mới, nhận `id`.
//! - `write` — gửi phím người dùng gõ vào shell.
//! - `resize` — đổi kích thước (số hàng/cột) khi khung terminal thay đổi.
//! - `kill`  — kết thúc phiên.
//!
//! Output của shell được đọc trong một thread nền riêng cho từng phiên và bắn
//! về frontend qua event `terminal-output`; khi tiến trình kết thúc bắn
//! `terminal-exit`. Đây là hạ tầng độc lập, không đụng tới bất kỳ nghiệp vụ nào
//! của các màn hình hiện có.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize};
use tauri::{AppHandle, Emitter};

use crate::models::terminal::{TerminalExit, TerminalOutput};

/// Tên event bắn mỗi khi có chunk output mới từ một phiên terminal.
pub const TERMINAL_OUTPUT_EVENT: &str = "terminal-output";
/// Tên event bắn khi tiến trình shell của một phiên kết thúc.
pub const TERMINAL_EXIT_EVENT: &str = "terminal-exit";

/// Một phiên terminal đang mở — giữ những handle cần cho write/resize/kill.
struct Session {
    /// Master của PTY, dùng để đổi kích thước (resize).
    master: Box<dyn MasterPty + Send>,
    /// Writer để đẩy dữ liệu người dùng gõ vào shell.
    writer: Box<dyn Write + Send>,
    /// Handle để kết thúc tiến trình con khi cần.
    killer: Box<dyn ChildKiller + Send + Sync>,
}

/// Registry toàn cục `id -> Session`. Khởi tạo lười bằng `OnceLock`.
fn sessions() -> &'static Mutex<HashMap<String, Session>> {
    static SESSIONS: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Bộ đếm sinh id phiên duy nhất trong vòng đời tiến trình app.
static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Chọn shell mặc định theo hệ điều hành nếu frontend không chỉ định.
fn default_shell() -> String {
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

/// Mã hoá base64 chuẩn (không phụ thuộc crate ngoài) cho dữ liệu byte thô đọc
/// từ PTY, để truyền an toàn qua ranh giới IPC dưới dạng chuỗi.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Mở một phiên terminal mới và trả về id của phiên.
///
/// - `cwd`   — thư mục làm việc ban đầu (rỗng → thư mục home của user).
/// - `shell` — đường dẫn shell (rỗng → shell mặc định của OS).
/// - `rows`/`cols` — kích thước khởi tạo của terminal.
pub fn spawn(
    app: AppHandle,
    cwd: Option<String>,
    shell: Option<String>,
    rows: u16,
    cols: u16,
) -> Result<String, String> {
    let pty_system = portable_pty::native_pty_system();
    let size = PtySize {
        rows: rows.max(1),
        cols: cols.max(1),
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = pty_system.openpty(size).map_err(|e| e.to_string())?;

    let shell_path = shell
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(default_shell);
    let mut cmd = CommandBuilder::new(shell_path);
    // Báo cho chương trình biết terminal hỗ trợ màu để render đẹp.
    cmd.env("TERM", "xterm-256color");

    let work_dir = cwd
        .filter(|d| !d.trim().is_empty())
        .or_else(|| dirs::home_dir().map(|p| p.to_string_lossy().to_string()));
    if let Some(dir) = work_dir {
        cmd.cwd(dir);
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| e.to_string())?;
    // Không cần giữ slave sau khi đã spawn — đóng để tránh giữ handle thừa.
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let killer = child.clone_killer();

    let id = format!("term-{}", COUNTER.fetch_add(1, Ordering::Relaxed));

    sessions().lock().unwrap().insert(
        id.clone(),
        Session {
            master: pair.master,
            writer,
            killer,
        },
    );

    // Đọc output bằng 2 thread để tránh nghẽn cầu IPC của webview:
    //  1. Reader thread: `read()` chặn (blocking) trên PTY, đẩy từng chunk thô
    //     qua channel — không tự emit.
    //  2. Emit thread: gom (coalesce) các chunk đến trong một cửa sổ ngắn thành
    //     MỘT event `terminal-output`. Khi shell vẽ lại prompt nhiều màu theo
    //     từng phím, hàng loạt chunk nhỏ được gộp lại → giảm mạnh số lần vượt
    //     cầu IPC, loại bỏ độ trễ gõ phím.
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();

    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        // Trả về → `tx` bị drop → channel đóng để emit thread biết đã EOF.
    });

    let emit_id = id.clone();
    std::thread::spawn(move || {
        use std::sync::mpsc::RecvTimeoutError;
        // Trần kích thước một lần gộp (64 KiB) để output lớn vẫn chảy mượt.
        const MAX_BATCH: usize = 64 * 1024;
        // Cửa sổ gom ngắn (8ms): đủ nhỏ để cảm giác tức thì, đủ để gộp 1 burst.
        const WINDOW: std::time::Duration = std::time::Duration::from_millis(8);

        loop {
            let mut batch = match rx.recv() {
                Ok(chunk) => chunk,
                Err(_) => break, // reader kết thúc và channel rỗng.
            };
            while batch.len() < MAX_BATCH {
                match rx.recv_timeout(WINDOW) {
                    Ok(mut more) => batch.append(&mut more),
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            let payload = TerminalOutput {
                id: emit_id.clone(),
                data: base64_encode(&batch),
            };
            if app.emit(TERMINAL_OUTPUT_EVENT, payload).is_err() {
                break;
            }
        }

        let code = child.wait().ok().map(|status| status.exit_code() as i32);
        sessions().lock().unwrap().remove(&emit_id);
        let _ = app.emit(
            TERMINAL_EXIT_EVENT,
            TerminalExit { id: emit_id, code },
        );
    });

    Ok(id)
}

/// Ghi dữ liệu (phím người dùng gõ) vào shell của phiên `id`.
pub fn write(id: &str, data: &str) -> Result<(), String> {
    let mut guard = sessions().lock().unwrap();
    let session = guard
        .get_mut(id)
        .ok_or_else(|| format!("Terminal session '{id}' không tồn tại."))?;
    session
        .writer
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    session.writer.flush().map_err(|e| e.to_string())
}

/// Đổi kích thước PTY của phiên `id` khi khung terminal thay đổi.
pub fn resize(id: &str, rows: u16, cols: u16) -> Result<(), String> {
    let guard = sessions().lock().unwrap();
    let session = guard
        .get(id)
        .ok_or_else(|| format!("Terminal session '{id}' không tồn tại."))?;
    session
        .master
        .resize(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())
}

/// Kết thúc phiên `id` (kill tiến trình + gỡ khỏi registry).
///
/// Không lỗi nếu phiên đã kết thúc từ trước — thao tác này là idempotent.
pub fn kill(id: &str) -> Result<(), String> {
    let session = sessions().lock().unwrap().remove(id);
    if let Some(mut session) = session {
        let _ = session.killer.kill();
    }
    Ok(())
}

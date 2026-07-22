//! Tiện ích kiểm tra kết nối mạng và lấy địa chỉ IP local.

use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::str::FromStr;
use std::time::Duration;

use ini::Ini;

use crate::utils::app_config;

/// Lấy địa chỉ IP local của máy bằng cách mở UDP socket tới Google DNS.
///
/// Không thực sự gửi dữ liệu — chỉ dùng `connect()` để OS xác định
/// network interface nào sẽ được sử dụng, từ đó lấy IP local.
pub fn local_ip_address() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Danh sách endpoint dùng để kiểm tra kết nối internet.
/// Các URL này trả về `204 No Content` — nhẹ và nhanh.
const CONNECTIVITY_PROBES: [&str; 2] = [
    "https://clients3.google.com/generate_204",
    "https://cp.cloudflare.com/generate_204",
];

/// Kiểm tra máy có thể kết nối internet hay không.
///
/// Thử gửi GET request tới các probe. Nếu bất kỳ probe nào phản hồi
/// (kể cả lỗi HTTP), coi như mạng hoạt động. Timeout: 5 giây.
pub async fn is_internet_reachable() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    for url in CONNECTIVITY_PROBES {
        if client.get(url).send().await.is_ok() {
            return true;
        }
    }

    false
}

/// Đọc danh sách IP nội bộ từ `[IPADRESS]` trong `config.ini`.
/// Trả về `Vec` rỗng nếu chưa cấu hình hoặc file không tồn tại.
fn load_ip_list() -> Vec<IpAddr> {
    let path = app_config::config_path();
    let ini = match Ini::load_from_file(&path) {
        Ok(ini) => ini,
        Err(_) => return Vec::new(),
    };

    let raw = match ini.get_from(Some("IPADRESS"), "IPADRESS_LIST") {
        Some(val) => val,
        None => return Vec::new(),
    };

    raw.split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                IpAddr::from_str(trimmed).ok()
            }
        })
        .collect()
}

/// Kiểm tra máy có nằm trong mạng nội bộ công ty (hoặc kết nối VPN) hay không.
///
/// Đọc danh sách IP từ `[IPADRESS].IPADRESS_LIST` trong `config.ini`.
/// Nếu chưa cấu hình → trả `true` (bỏ qua kiểm tra).
/// Nếu đã cấu hình → thử TCP connect tới từng IP; bất kỳ IP nào phản hồi
/// (kể cả connection-refused) nghĩa là mạng nội bộ đang khả dụng.
pub async fn is_internal_reachable() -> bool {
    let ips = load_ip_list();
    if ips.is_empty() {
        return true;
    }

    for ip in ips {
        let reachable = tokio::task::spawn_blocking(move || {
            let addr = SocketAddr::new(ip, 80);
            match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
                Ok(_) => true,
                Err(e) => e.kind() == std::io::ErrorKind::ConnectionRefused,
            }
        })
        .await
        .unwrap_or(false);

        if reachable {
            return true;
        }
    }

    false
}

//! Tiện ích kiểm tra kết nối mạng và lấy địa chỉ IP local.

use std::net::{IpAddr, TcpStream, ToSocketAddrs, UdpSocket};
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

/// Đọc danh sách IP public công ty từ `[IPADRESS].IPADRESS_LIST` trong `config.ini`.
///
/// Đây là các IP egress mà công ty cấp để truy xuất ra ngoài — khi máy ở trong
/// mạng công ty (hoặc VPN full-tunnel), IP public của máy sẽ là một trong số này.
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

/// Danh sách endpoint trả về IP public (egress) của máy dưới dạng text thuần.
const PUBLIC_IP_PROBES: [&str; 2] = [
    "https://checkip.amazonaws.com",
    "https://api.ipify.org",
];

/// Lấy địa chỉ IP public (egress) hiện tại của máy — tức IP mà internet nhìn thấy.
///
/// Thử lần lượt các endpoint, trả về IP đầu tiên parse được. Timeout: 5 giây.
/// Trả `None` nếu không truy vấn được (mất mạng / timeout / phản hồi không hợp lệ).
pub async fn public_ip() -> Option<IpAddr> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;

    for url in PUBLIC_IP_PROBES {
        if let Ok(resp) = client.get(url).send().await {
            if let Ok(text) = resp.text().await {
                if let Ok(ip) = IpAddr::from_str(text.trim()) {
                    return Some(ip);
                }
            }
        }
    }

    None
}

/// Xác nhận IP public hiện tại có nằm trong danh sách IP công ty cấp
/// (`[IPADRESS].IPADRESS_LIST`) hay không.
///
/// - `Some(true)`  → IP public khớp IP công ty → đang ở trong mạng công ty / VPN.
/// - `Some(false)` → lấy được IP public nhưng KHÔNG khớp → không ở mạng công ty.
/// - `None`        → chưa cấu hình danh sách IP hoặc không lấy được IP public
///   → không xác định được (không nên dùng để kết luận).
pub async fn is_company_egress_ip() -> Option<bool> {
    let ips = load_ip_list();
    if ips.is_empty() {
        return None;
    }
    let current = public_ip().await?;
    Some(ips.contains(&current))
}

/// Kiểm tra máy có nằm trong mạng công ty (hoặc kết nối VPN) hay không.
///
/// Dựa trên xác nhận IP public: chỉ trả `false` khi chắc chắn IP public KHÔNG
/// thuộc dải công ty. Nếu chưa cấu hình IP hoặc không lấy được IP public thì
/// trả `true` (không chặn — tránh báo nhầm khi không đủ dữ liệu).
pub async fn is_internal_reachable() -> bool {
    !matches!(is_company_egress_ip().await, Some(false))
}

/// Kiểm tra có mở được kết nối TCP tới `host:port` hay không.
///
/// Dùng để phân biệt lỗi mạng (chưa vào mạng nội bộ / chưa VPN) với lỗi
/// dịch vụ/cấu hình khi kết nối database thất bại:
/// - `Ok` hoặc `ConnectionRefused` → host tồn tại trong mạng (chỉ là cổng
///   không mở / dịch vụ chưa chạy) → coi như **reachable**.
/// - Timeout / no route / DNS không resolve (thường gặp khi off-VPN với host
///   nội bộ) → **không reachable**.
pub async fn is_tcp_reachable(host: &str, port: u16) -> bool {
    let host = host.trim().to_string();
    if host.is_empty() {
        return false;
    }

    tokio::task::spawn_blocking(move || {
        let addrs = match (host.as_str(), port).to_socket_addrs() {
            Ok(addrs) => addrs,
            // Không resolve được DNS (vd hostname nội bộ khi chưa VPN).
            Err(_) => return false,
        };

        for addr in addrs {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
                Ok(_) => return true,
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => return true,
                Err(_) => continue,
            }
        }
        false
    })
    .await
    .unwrap_or(false)
}

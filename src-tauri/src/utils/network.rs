//! Tiện ích kiểm tra kết nối mạng và lấy địa chỉ IP local.

use std::net::UdpSocket;
use std::time::Duration;

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

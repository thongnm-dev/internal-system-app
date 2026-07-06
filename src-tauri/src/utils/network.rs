use std::net::UdpSocket;
use std::time::Duration;

pub fn local_ip_address() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Lightweight probes that return an empty `204 No Content` response.
/// Reaching any of them means the machine can talk to the internet.
const CONNECTIVITY_PROBES: [&str; 2] = [
    "https://clients3.google.com/generate_204",
    "https://cp.cloudflare.com/generate_204",
];

/// Returns `true` when at least one connectivity probe replies within the
/// timeout. Any HTTP response (even an error status) proves we reached a
/// server, so a successful send is enough to consider the network online.
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

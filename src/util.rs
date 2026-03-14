/// Build a WebSocket URL from an HTTP server URL and a path suffix.
/// http:// → ws://, https:// → wss://
pub fn build_ws_url(server_url: &str, path: &str) -> String {
    let (scheme, host) = if let Some(host) = server_url.strip_prefix("https://") {
        ("wss", host)
    } else if let Some(host) = server_url.strip_prefix("http://") {
        ("ws", host)
    } else {
        ("ws", server_url)
    };
    format!("{}://{}{}", scheme, host, path)
}

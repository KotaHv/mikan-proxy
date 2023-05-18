use axum::{
    extract::ConnectInfo,
    http::{header, HeaderMap, Request},
};

pub fn get_ip<B>(req: &Request<B>) -> String {
    let headers = req.headers();
    if let Some(ip) = get_header(headers, "cf-connecting-ip") {
        return ip;
    }
    if let Some(ip) = get_header(headers, "X-Forwarded-For") {
        if let Some(ip) = ip.split_once(",") {
            return ip.0.to_string();
        }
        return ip;
    }
    if let Some(ip) = get_header(headers, "X-Real-IP") {
        return ip;
    }
    if let Some(ConnectInfo(ip)) = req.extensions().get::<ConnectInfo<std::net::SocketAddr>>() {
        return ip.to_string();
    }
    "-".to_string()
}

pub fn get_header(headers: &HeaderMap, key: impl header::AsHeaderName) -> Option<String> {
    if let Some(header) = headers.get(key) {
        if let Ok(header) = header.to_str() {
            return Some(header.to_string());
        }
    }
    None
}

pub fn get_ua(headers: &HeaderMap) -> String {
    match get_header(headers, header::USER_AGENT) {
        Some(ua) => ua,
        None => "-".to_string(),
    }
}

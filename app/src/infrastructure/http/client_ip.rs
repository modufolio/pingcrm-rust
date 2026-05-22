use axum::http::HeaderMap;
use std::net::{IpAddr, SocketAddr};

pub fn extract_client_ip(headers: &HeaderMap, connection_info: SocketAddr) -> IpAddr {
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    connection_info.ip()
}

pub struct ClientIp(pub IpAddr);

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_extract_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.1, 198.51.100.1"),
        );
        let connection_info = SocketAddr::from(([127, 0, 0, 1], 8080));

        let ip = extract_client_ip(&headers, connection_info);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)));
    }

    #[test]
    fn test_extract_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.42"));
        let connection_info = SocketAddr::from(([127, 0, 0, 1], 8080));

        let ip = extract_client_ip(&headers, connection_info);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 42)));
    }

    #[test]
    fn test_fallback_to_connection_info() {
        let headers = HeaderMap::new();
        let connection_info = SocketAddr::from(([192, 168, 1, 100], 8080));

        let ip = extract_client_ip(&headers, connection_info);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
    }

    #[test]
    fn test_x_forwarded_for_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1"));
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.2"));
        let connection_info = SocketAddr::from(([127, 0, 0, 1], 8080));

        let ip = extract_client_ip(&headers, connection_info);

        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)));
    }
}

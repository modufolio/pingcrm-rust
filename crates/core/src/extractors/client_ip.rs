use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub IpAddr);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(forwarded_for) = parts.headers.get("x-forwarded-for") {
            if let Ok(header_value) = forwarded_for.to_str() {
                if let Some(first_ip) = header_value.split(',').next() {
                    if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                        return Ok(ClientIp(ip));
                    }
                }
            }
        }

        if let Some(real_ip) = parts.headers.get("x-real-ip") {
            if let Ok(header_value) = real_ip.to_str() {
                if let Ok(ip) = header_value.trim().parse::<IpAddr>() {
                    return Ok(ClientIp(ip));
                }
            }
        }

        if let Some(forwarded) = parts.headers.get("forwarded") {
            if let Ok(header_value) = forwarded.to_str() {
                for part in header_value.split(';') {
                    if let Some(for_value) = part.trim().strip_prefix("for=") {
                        let ip_str = for_value
                            .trim_matches('"')
                            .trim_start_matches('[')
                            .trim_end_matches(']');

                        if let Ok(ip) = ip_str.parse::<IpAddr>() {
                            return Ok(ClientIp(ip));
                        }
                    }
                }
            }
        }

        if let Some(connect_info) = parts
            .extensions
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        {
            return Ok(ClientIp(connect_info.0.ip()));
        }

        Ok(ClientIp(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, Request};
    use std::net::Ipv4Addr;

    fn make_parts() -> Parts {
        let req = Request::builder().body(()).unwrap();
        let (parts, _) = req.into_parts();
        parts
    }

    #[tokio::test]
    async fn test_x_forwarded_for() {
        let mut parts = make_parts();
        parts.headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.195, 70.41.3.18, 150.172.238.178"),
        );

        let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 195)));
    }

    #[tokio::test]
    async fn test_x_real_ip() {
        let mut parts = make_parts();
        parts
            .headers
            .insert("x-real-ip", HeaderValue::from_static("192.168.1.1"));

        let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[tokio::test]
    async fn test_forwarded_header() {
        let mut parts = make_parts();
        parts.headers.insert(
            "forwarded",
            HeaderValue::from_static("for=192.0.2.60;proto=http;by=203.0.113.43"),
        );

        let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 60)));
    }

    #[tokio::test]
    async fn test_ipv6() {
        let mut parts = make_parts();
        parts
            .headers
            .insert("x-real-ip", HeaderValue::from_static("2001:db8::1"));

        let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(ip, IpAddr::V6("2001:db8::1".parse().unwrap()));
    }
}

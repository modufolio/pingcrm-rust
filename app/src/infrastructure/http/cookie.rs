use axum::http::{header, HeaderMap, HeaderValue};
use std::time::Duration;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,

    Lax,

    None,
}

impl SameSite {
    fn as_str(&self) -> &str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cookie {
    name: String,
    value: String,
    domain: Option<String>,
    path: Option<String>,
    max_age: Option<Duration>,
    expires: Option<OffsetDateTime>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            max_age: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    pub fn session(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(name, value)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
    }

    pub fn permanent(name: impl Into<String>, value: impl Into<String>, max_age: Duration) -> Self {
        Self::new(name, value)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(max_age)
    }

    pub fn deleted(name: impl Into<String>) -> Self {
        Self::new(name, "")
            .max_age(Duration::from_secs(0))
            .expires(OffsetDateTime::UNIX_EPOCH)
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn max_age(mut self, duration: Duration) -> Self {
        self.max_age = Some(duration);
        self
    }

    pub fn expires(mut self, timestamp: OffsetDateTime) -> Self {
        self.expires = Some(timestamp);
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    pub fn build(self) -> String {
        let mut cookie = format!("{}={}", self.name, self.value);

        if let Some(domain) = self.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }

        if let Some(path) = self.path {
            cookie.push_str(&format!("; Path={}", path));
        }

        if let Some(max_age) = self.max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age.as_secs()));
        }

        if let Some(expires) = self.expires {
            if let Ok(formatted) = expires.format(&Rfc2822) {
                cookie.push_str(&format!("; Expires={}", formatted));
            }
        }

        if self.secure {
            cookie.push_str("; Secure");
        }

        if self.http_only {
            cookie.push_str("; HttpOnly");
        }

        if let Some(same_site) = self.same_site {
            cookie.push_str(&format!("; SameSite={}", same_site.as_str()));
        }

        cookie
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Default, Clone)]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, cookie: Cookie) {
        self.cookies.push(cookie);
    }

    pub fn remove(&mut self, name: impl Into<String>) {
        self.cookies.push(Cookie::deleted(name));
    }

    pub fn headers(&self) -> Vec<(header::HeaderName, HeaderValue)> {
        self.cookies
            .iter()
            .filter_map(|cookie| {
                let value = cookie.clone().build();
                HeaderValue::from_str(&value)
                    .ok()
                    .map(|v| (header::SET_COOKIE, v))
            })
            .collect()
    }

    pub fn apply_to_headers(&self, headers: &mut HeaderMap) {
        for (name, value) in self.headers() {
            headers.append(name, value);
        }
    }
}

pub struct CookieParser;

impl CookieParser {
    pub fn parse(headers: &HeaderMap) -> Vec<(String, String)> {
        headers
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(Self::parse_cookie_header)
            .unwrap_or_default()
    }

    pub fn get(headers: &HeaderMap, name: &str) -> Option<String> {
        Self::parse(headers)
            .into_iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
    }

    fn parse_cookie_header(header: &str) -> Vec<(String, String)> {
        header
            .split(';')
            .filter_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                let name = parts.next()?.trim().to_string();
                let value = parts.next()?.trim().to_string();
                Some((name, value))
            })
            .collect()
    }
}

pub trait ResponseCookies {
    fn cookie(self, cookie: Cookie) -> Self;

    fn cookies(self, cookies: CookieJar) -> Self;
}

impl<T> ResponseCookies for axum::response::Response<T>
where
    T: axum::body::HttpBody,
{
    fn cookie(mut self, cookie: Cookie) -> Self {
        let value = cookie.build();
        if let Ok(header_value) = HeaderValue::from_str(&value) {
            self.headers_mut().append(header::SET_COOKIE, header_value);
        }
        self
    }

    fn cookies(mut self, cookies: CookieJar) -> Self {
        cookies.apply_to_headers(self.headers_mut());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_basic() {
        let cookie = Cookie::new("name", "value").build();
        assert_eq!(cookie, "name=value");
    }

    #[test]
    fn test_cookie_with_options() {
        let cookie = Cookie::new("session", "abc123")
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .build();

        assert!(cookie.contains("session=abc123"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Strict"));
    }

    #[test]
    fn test_cookie_max_age() {
        let cookie = Cookie::new("test", "value")
            .max_age(Duration::from_secs(3600))
            .build();

        assert!(cookie.contains("Max-Age=3600"));
    }

    #[test]
    fn test_cookie_deleted() {
        let cookie = Cookie::deleted("old_cookie").build();
        assert!(cookie.contains("old_cookie="));
        assert!(cookie.contains("Max-Age=0"));
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();
        jar.add(Cookie::new("cookie1", "value1"));
        jar.add(Cookie::new("cookie2", "value2"));

        let headers = jar.headers();
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_cookie_parser() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("session=abc123; user=john"),
        );

        let cookies = CookieParser::parse(&headers);
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0], ("session".to_string(), "abc123".to_string()));
        assert_eq!(cookies[1], ("user".to_string(), "john".to_string()));
    }

    #[test]
    fn test_cookie_parser_get() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("session=abc123; user=john"),
        );

        assert_eq!(
            CookieParser::get(&headers, "session"),
            Some("abc123".to_string())
        );
        assert_eq!(
            CookieParser::get(&headers, "user"),
            Some("john".to_string())
        );
        assert_eq!(CookieParser::get(&headers, "missing"), None);
    }

    #[test]
    fn test_same_site_values() {
        assert_eq!(SameSite::Strict.as_str(), "Strict");
        assert_eq!(SameSite::Lax.as_str(), "Lax");
        assert_eq!(SameSite::None.as_str(), "None");
    }

    #[test]
    fn test_cookie_session() {
        let cookie = Cookie::session("sid", "xyz").build();
        assert!(cookie.contains("sid=xyz"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Lax"));
    }

    #[test]
    fn test_cookie_permanent() {
        let cookie = Cookie::permanent("remember", "token", Duration::from_secs(86400)).build();
        assert!(cookie.contains("remember=token"));
        assert!(cookie.contains("Max-Age=86400"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
    }
}

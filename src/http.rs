use std::{fmt, str};

use async_trait::async_trait;

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("name", &self.name)
            .field("value", &"***")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Request {
    method: Method,
    url: String,
    headers: Vec<Header>,
    query_params: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Vec::new(),
            query_params: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url)
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push(Header::new(name, value));
        self
    }

    pub fn with_query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((name.into(), value.into()));
        self
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn headers(&self) -> &[Header] {
        &self.headers
    }

    pub fn query_params(&self) -> &[(String, String)] {
        &self.query_params
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field(
                "query_params",
                &format_args!("<{} params>", self.query_params.len()),
            )
            .field("body", &format_args!("<{} bytes>", self.body.len()))
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Response {
    status_code: u16,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn with_headers(mut self, headers: impl IntoIterator<Item = Header>) -> Self {
        self.headers = headers.into_iter().collect();
        self
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn headers(&self) -> &[Header] {
        &self.headers
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|header| header.name().eq_ignore_ascii_case(name))
            .map(Header::value)
    }

    pub fn body_str(&self) -> Result<&str> {
        str::from_utf8(&self.body)
            .map_err(|error| Error::parse(format!("response body is not UTF-8: {error}")))
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("status_code", &self.status_code)
            .field("headers", &format_args!("<{} headers>", self.headers.len()))
            .field("body", &format_args!("<{} bytes>", self.body.len()))
            .finish()
    }
}

#[async_trait]
pub trait HttpClient {
    async fn send(&self, request: Request) -> Result<Response>;
}

#[cfg(feature = "reqwest-client")]
#[derive(Clone, Debug)]
pub struct ReqwestHttpClient {
    inner: reqwest::Client,
}

#[cfg(feature = "reqwest-client")]
impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self::with_client(reqwest::Client::new())
    }

    pub fn with_client(inner: reqwest::Client) -> Self {
        Self { inner }
    }

    fn build_request(&self, request: Request) -> Result<reqwest::Request> {
        let mut builder = match request.method() {
            Method::Get => self.inner.get(request.url()),
            Method::Post => self.inner.post(request.url()),
        };

        for header in request.headers() {
            builder = builder.header(header.name(), header.value());
        }

        builder
            .query(request.query_params())
            .body(request.body().to_vec())
            .build()
            .map_err(|error| Error::http(format!("failed to build HTTP request: {error}")))
    }
}

#[cfg(feature = "reqwest-client")]
impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "reqwest-client")]
#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn send(&self, request: Request) -> Result<Response> {
        let request = self.build_request(request)?;
        let response = self
            .inner
            .execute(request)
            .await
            .map_err(|error| Error::http(format!("HTTP request failed: {error}")))?;
        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| Header::new(name.as_str(), value))
            })
            .collect::<Vec<_>>();
        let body = response
            .bytes()
            .await
            .map_err(|error| Error::http(format!("failed to read HTTP response body: {error}")))?;

        Ok(Response::new(status_code, body.to_vec()).with_headers(headers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_debug_does_not_expose_body_or_header_value() {
        let request = Request::post("https://example.com")
            .with_header("authorization", "Bearer access-token")
            .with_query_param("CANO", "12345678")
            .with_body("app-secret-value");

        let debug = format!("{request:?}");

        assert!(!debug.contains("Bearer access-token"));
        assert!(!debug.contains("12345678"));
        assert!(!debug.contains("app-secret-value"));
        assert!(debug.contains("<1 params>"));
        assert!(debug.contains("<16 bytes>"));
    }

    #[test]
    fn response_debug_does_not_expose_body() {
        let response = Response::new(200, "access-token-value")
            .with_headers([Header::new("authorization", "Bearer access-token")]);

        let debug = format!("{response:?}");

        assert!(!debug.contains("access-token-value"));
        assert!(!debug.contains("Bearer access-token"));
        assert!(debug.contains("<1 headers>"));
        assert!(debug.contains("<18 bytes>"));
    }

    #[test]
    fn response_header_lookup_is_case_insensitive() {
        let response = Response::new(200, "{}").with_headers([Header::new("tr_cont", "M")]);

        assert_eq!(response.header("TR_CONT"), Some("M"));
    }

    #[cfg(feature = "reqwest-client")]
    #[test]
    fn reqwest_client_builds_post_request() {
        let client = ReqwestHttpClient::new();
        let request = Request::post("https://example.com/token")
            .with_header("content-type", "application/json")
            .with_body(r#"{"grant_type":"client_credentials"}"#);

        let request = client.build_request(request).unwrap();

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(request.url().as_str(), "https://example.com/token");
        assert_eq!(request.headers()["content-type"], "application/json");
        assert_eq!(
            request.body().and_then(|body| body.as_bytes()),
            Some(br#"{"grant_type":"client_credentials"}"#.as_slice())
        );
    }

    #[cfg(feature = "reqwest-client")]
    #[test]
    fn reqwest_client_builds_get_request_with_query() {
        let client = ReqwestHttpClient::new();
        let request = Request::get("https://example.com/quote")
            .with_query_param("FID_COND_MRKT_DIV_CODE", "J")
            .with_query_param("FID_INPUT_ISCD", "005930");

        let request = client.build_request(request).unwrap();

        assert_eq!(request.method(), reqwest::Method::GET);
        assert_eq!(
            request.url().as_str(),
            "https://example.com/quote?FID_COND_MRKT_DIV_CODE=J&FID_INPUT_ISCD=005930"
        );
    }
}

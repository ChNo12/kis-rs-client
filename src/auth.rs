use std::fmt;

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::config::{Credentials, SecretString};
use crate::error::{Error, Result};
use crate::http::{HttpClient, Request, Response};
use crate::rest::{ApiResponseStatus, CONTENT_TYPE_JSON};

pub const TOKEN_PATH: &str = "/oauth2/tokenP";
pub const APPROVAL_PATH: &str = "/oauth2/Approval";
pub const CLIENT_CREDENTIALS_GRANT_TYPE: &str = "client_credentials";
pub const AUTHORIZATION_HEADER: &str = "authorization";

#[derive(Eq, PartialEq, Serialize)]
pub struct TokenIssueRequest<'a> {
    pub grant_type: &'static str,
    #[serde(rename = "appkey")]
    pub app_key: &'a str,
    #[serde(rename = "appsecret")]
    pub app_secret: &'a str,
}

impl<'a> TokenIssueRequest<'a> {
    pub fn new(credentials: &'a Credentials) -> Self {
        Self {
            grant_type: CLIENT_CREDENTIALS_GRANT_TYPE,
            app_key: credentials.app_key.expose_secret(),
            app_secret: credentials.app_secret.expose_secret(),
        }
    }
}

impl fmt::Debug for TokenIssueRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenIssueRequest")
            .field("grant_type", &self.grant_type)
            .field("app_key", &"***")
            .field("app_secret", &"***")
            .finish()
    }
}

#[derive(Eq, PartialEq, Serialize)]
pub struct ApprovalKeyIssueRequest<'a> {
    pub grant_type: &'static str,
    #[serde(rename = "appkey")]
    pub app_key: &'a str,
    #[serde(rename = "secretkey")]
    pub secret_key: &'a str,
}

impl<'a> ApprovalKeyIssueRequest<'a> {
    pub fn new(credentials: &'a Credentials) -> Self {
        Self {
            grant_type: CLIENT_CREDENTIALS_GRANT_TYPE,
            app_key: credentials.app_key.expose_secret(),
            secret_key: credentials.app_secret.expose_secret(),
        }
    }
}

impl fmt::Debug for ApprovalKeyIssueRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApprovalKeyIssueRequest")
            .field("grant_type", &self.grant_type)
            .field("app_key", &"***")
            .field("secret_key", &"***")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct AccessToken(SecretString);

impl AccessToken {
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecretString::new(value))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }

    pub fn bearer_authorization(&self) -> String {
        format!("Bearer {}", self.expose_secret())
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccessToken(***)")
    }
}

impl<'de> Deserialize<'de> for AccessToken {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SecretString::deserialize(deserializer).map(Self)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ApprovalKey(SecretString);

impl ApprovalKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecretString::new(value))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for ApprovalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApprovalKey(***)")
    }
}

impl<'de> Deserialize<'de> for ApprovalKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SecretString::deserialize(deserializer).map(Self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub token_type: String,
    pub expires_in: u64,
    pub access_token_token_expired: Option<String>,
    pub refresh_token: Option<SecretString>,
    pub refresh_token_expires_in: Option<u64>,
    pub refresh_token_token_expired: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ApprovalKeyResponse {
    pub approval_key: ApprovalKey,
}

pub(crate) async fn issue_token<T: HttpClient>(client: &Client<T>) -> Result<TokenResponse> {
    let token_request = TokenIssueRequest::new(&client.config().credentials);
    let body = serde_json::to_vec(&token_request)
        .map_err(|error| Error::parse(format!("failed to serialize token request: {error}")))?;

    let request = Request::post(client.rest_endpoint_url(TOKEN_PATH))
        .with_header("content-type", CONTENT_TYPE_JSON)
        .with_body(body);

    let response = client.http_client().send(request).await?;

    parse_token_response(response)
}

pub(crate) async fn issue_approval_key<T: HttpClient>(
    client: &Client<T>,
) -> Result<ApprovalKeyResponse> {
    let approval_request = ApprovalKeyIssueRequest::new(&client.config().credentials);
    let body = serde_json::to_vec(&approval_request).map_err(|error| {
        Error::parse(format!("failed to serialize approval key request: {error}"))
    })?;

    let request = Request::post(client.rest_endpoint_url(APPROVAL_PATH))
        .with_header("content-type", CONTENT_TYPE_JSON)
        .with_body(body);

    let response = client.http_client().send(request).await?;

    parse_approval_key_response(response)
}

fn parse_token_response(response: Response) -> Result<TokenResponse> {
    if let Some(status) = ApiResponseStatus::from_body(response.body())
        && !status.is_success()
    {
        return Err(status.into_api_error(status_fallback_message(&response)));
    }

    if !response.is_success() {
        return Err(Error::api(
            None,
            format!("unexpected HTTP status: {}", response.status_code()),
        ));
    }

    serde_json::from_slice(response.body())
        .map_err(|error| Error::parse(format!("failed to parse token response: {error}")))
}

fn parse_approval_key_response(response: Response) -> Result<ApprovalKeyResponse> {
    if let Some(status) = ApiResponseStatus::from_body(response.body())
        && !status.is_success()
    {
        return Err(status.into_api_error(status_fallback_message(&response)));
    }

    if !response.is_success() {
        return Err(Error::api(
            None,
            format!("unexpected HTTP status: {}", response.status_code()),
        ));
    }

    serde_json::from_slice(response.body())
        .map_err(|error| Error::parse(format!("failed to parse approval key response: {error}")))
}

fn status_fallback_message(response: &Response) -> String {
    if response.is_success() {
        "KIS API request failed".to_string()
    } else {
        format!("unexpected HTTP status: {}", response.status_code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Environment};
    use crate::http::{Method, Response};
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Debug)]
    struct MockHttpClient {
        response: Response,
        requests: Mutex<Vec<Request>>,
    }

    impl MockHttpClient {
        fn new(response: Response) -> Self {
            Self {
                response,
                requests: Mutex::new(Vec::new()),
            }
        }

        fn requests(&self) -> Vec<Request> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl HttpClient for &MockHttpClient {
        async fn send(&self, request: Request) -> Result<Response> {
            self.requests.lock().unwrap().push(request);
            Ok(self.response.clone())
        }
    }

    use serde_json::json;

    #[test]
    fn token_issue_request_serializes_kis_fields() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let request = TokenIssueRequest::new(&credentials);

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(
            value,
            json!({
                "grant_type": "client_credentials",
                "appkey": "app-key",
                "appsecret": "app-secret"
            })
        );
    }

    #[test]
    fn token_issue_request_debug_does_not_expose_credentials() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let request = TokenIssueRequest::new(&credentials);

        let debug = format!("{request:?}");

        assert!(!debug.contains("app-key"));
        assert!(!debug.contains("app-secret"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn approval_key_issue_request_serializes_kis_fields() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let request = ApprovalKeyIssueRequest::new(&credentials);

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(
            value,
            json!({
                "grant_type": "client_credentials",
                "appkey": "app-key",
                "secretkey": "app-secret"
            })
        );
    }

    #[test]
    fn approval_key_issue_request_debug_does_not_expose_credentials() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let request = ApprovalKeyIssueRequest::new(&credentials);

        let debug = format!("{request:?}");

        assert!(!debug.contains("app-key"));
        assert!(!debug.contains("app-secret"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn token_response_deserializes_access_token_body() {
        let body = r#"{
            "access_token": "access-token-value",
            "access_token_token_expired": "2026-05-11 00:00:00",
            "token_type": "Bearer",
            "expires_in": 86400
        }"#;

        let response: TokenResponse = serde_json::from_str(body).unwrap();

        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 86400);
        assert_eq!(response.access_token.expose_secret(), "access-token-value");
        assert_eq!(response.refresh_token, None);
    }

    #[test]
    fn token_response_debug_does_not_expose_tokens() {
        let body = r#"{
            "access_token": "access-token-value",
            "refresh_token": "refresh-token-value",
            "access_token_token_expired": "2026-05-11 00:00:00",
            "refresh_token_token_expired": "2026-05-12 00:00:00",
            "refresh_token_expires_in": 172800,
            "token_type": "Bearer",
            "expires_in": 86400
        }"#;

        let response: TokenResponse = serde_json::from_str(body).unwrap();
        let debug = format!("{response:?}");

        assert!(!debug.contains("access-token-value"));
        assert!(!debug.contains("refresh-token-value"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn approval_key_response_debug_does_not_expose_key() {
        let body = r#"{
            "approval_key": "approval-key-value"
        }"#;

        let response: ApprovalKeyResponse = serde_json::from_str(body).unwrap();
        let debug = format!("{response:?}");

        assert_eq!(response.approval_key.expose_secret(), "approval-key-value");
        assert!(!debug.contains("approval-key-value"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn access_token_builds_bearer_authorization_without_debug_leak() {
        let access_token = AccessToken::new("access-token-value");

        assert_eq!(
            access_token.bearer_authorization(),
            "Bearer access-token-value"
        );

        let debug = format!("{access_token:?}");

        assert!(!debug.contains("access-token-value"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn request_with_bearer_authorization_does_not_debug_leak() {
        let access_token = AccessToken::new("access-token-value");
        let request = Request::post("https://example.com")
            .with_header(AUTHORIZATION_HEADER, access_token.bearer_authorization());

        let debug = format!("{request:?}");

        assert!(!debug.contains("access-token-value"));
        assert!(debug.contains("***"));
    }

    #[tokio::test]
    async fn issue_token_sends_kis_token_request() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);
        let http_client = MockHttpClient::new(Response::new(
            200,
            r#"{
                "access_token": "access-token-value",
                "access_token_token_expired": "2026-05-11 00:00:00",
                "token_type": "Bearer",
                "expires_in": 86400
            }"#,
        ));
        let client = Client::new(config, &http_client);

        let token = client.issue_token().await.unwrap();

        assert_eq!(token.access_token.expose_secret(), "access-token-value");
        assert_eq!(token.token_type, "Bearer");

        let requests = http_client.requests();
        assert_eq!(requests.len(), 1);

        let request = &requests[0];
        assert_eq!(request.method(), Method::Post);
        assert_eq!(
            request.url(),
            "https://openapivts.koreainvestment.com:29443/oauth2/tokenP"
        );
        assert_eq!(request.headers()[0].name(), "content-type");
        assert_eq!(request.headers()[0].value(), "application/json");

        let body: serde_json::Value = serde_json::from_slice(request.body()).unwrap();
        assert_eq!(
            body,
            json!({
                "grant_type": "client_credentials",
                "appkey": "app-key",
                "appsecret": "app-secret"
            })
        );
    }

    #[tokio::test]
    async fn issue_approval_key_sends_kis_approval_request() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);
        let http_client = MockHttpClient::new(Response::new(
            200,
            r#"{
                "approval_key": "approval-key-value"
            }"#,
        ));
        let client = Client::new(config, &http_client);

        let response = issue_approval_key(&client).await.unwrap();

        assert_eq!(response.approval_key.expose_secret(), "approval-key-value");

        let requests = http_client.requests();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.method(), Method::Post);
        assert_eq!(
            request.url(),
            "https://openapivts.koreainvestment.com:29443/oauth2/Approval"
        );
        assert!(request.headers().iter().any(|header| {
            header.name() == "content-type" && header.value() == CONTENT_TYPE_JSON
        }));

        let body: serde_json::Value = serde_json::from_slice(request.body()).unwrap();
        assert_eq!(
            body,
            json!({
                "grant_type": "client_credentials",
                "appkey": "app-key",
                "secretkey": "app-secret"
            })
        );
    }

    #[tokio::test]
    async fn issue_token_rejects_non_success_http_status() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);
        let http_client = MockHttpClient::new(Response::new(401, "denied"));
        let client = Client::new(config, &http_client);

        let error = client.issue_token().await.unwrap_err();

        assert_eq!(
            error,
            Error::api(None, "unexpected HTTP status: 401".to_string())
        );
    }

    #[tokio::test]
    async fn issue_token_uses_kis_error_body_on_non_success_http_status() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);
        let http_client = MockHttpClient::new(Response::new(
            401,
            r#"{
                "rt_cd": "1",
                "msg_cd": "EGW00123",
                "msg1": "invalid app key"
            }"#,
        ));
        let client = Client::new(config, &http_client);

        let error = client.issue_token().await.unwrap_err();

        assert_eq!(
            error,
            Error::api(Some("EGW00123".to_string()), "invalid app key")
        );
    }

    #[tokio::test]
    async fn issue_token_rejects_kis_error_body_with_success_http_status() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);
        let http_client = MockHttpClient::new(Response::new(
            200,
            r#"{
                "rt_cd": "1",
                "msg_cd": "EGW00123",
                "msg1": "invalid app key"
            }"#,
        ));
        let client = Client::new(config, &http_client);

        let error = client.issue_token().await.unwrap_err();

        assert_eq!(
            error,
            Error::api(Some("EGW00123".to_string()), "invalid app key")
        );
    }

    #[tokio::test]
    async fn issue_token_rejects_invalid_json_response() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);
        let http_client = MockHttpClient::new(Response::new(200, "not-json"));
        let client = Client::new(config, &http_client);

        let error = client.issue_token().await.unwrap_err();

        assert!(matches!(error, Error::Parse { .. }));
    }
}

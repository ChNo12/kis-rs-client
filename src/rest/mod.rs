use serde::Deserialize;

pub mod domestic_stock;
pub mod overseas_stock;
mod pagination;

pub use pagination::{PageCollection, PageLimit, PageStopReason};

pub const CONTENT_TYPE_JSON: &str = "application/json";
pub const SUCCESS_RT_CD: &str = "0";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ApiResponseStatus {
    pub rt_cd: Option<String>,
    pub msg_cd: Option<String>,
    pub msg1: Option<String>,
}

impl ApiResponseStatus {
    pub fn from_body(body: &[u8]) -> Option<Self> {
        let status = serde_json::from_slice::<Self>(body).ok()?;

        if status.rt_cd.is_none() && status.msg_cd.is_none() && status.msg1.is_none() {
            return None;
        }

        Some(status)
    }

    pub fn is_success(&self) -> bool {
        self.rt_cd.as_deref() == Some(SUCCESS_RT_CD)
    }

    pub fn into_api_error(self, fallback_message: impl Into<String>) -> crate::Error {
        let code = self.msg_cd.filter(|code| !code.is_empty());
        let message = self
            .msg1
            .filter(|message| !message.is_empty())
            .unwrap_or_else(|| fallback_message.into());

        crate::Error::api(code, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn parses_kis_error_status() {
        let body = br#"{
            "rt_cd": "1",
            "msg_cd": "EGW00123",
            "msg1": "invalid app key"
        }"#;

        let status = ApiResponseStatus::from_body(body).unwrap();

        assert!(!status.is_success());
        assert_eq!(
            status.into_api_error("fallback"),
            Error::api(Some("EGW00123".to_string()), "invalid app key")
        );
    }

    #[test]
    fn parses_kis_success_status() {
        let body = br#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok"
        }"#;

        let status = ApiResponseStatus::from_body(body).unwrap();

        assert!(status.is_success());
    }

    #[test]
    fn ignores_body_without_kis_status_fields() {
        let body = br#"{
            "access_token": "access-token-value",
            "token_type": "Bearer",
            "expires_in": 86400
        }"#;

        assert_eq!(ApiResponseStatus::from_body(body), None);
    }
}

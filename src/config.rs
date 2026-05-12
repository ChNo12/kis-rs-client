use std::fmt;

use serde::Deserialize;

use crate::error::{Error, Result};

pub const REAL_REST_BASE_URL: &str = "https://openapi.koreainvestment.com:9443";
pub const MOCK_REST_BASE_URL: &str = "https://openapivts.koreainvestment.com:29443";
pub const REAL_WEBSOCKET_BASE_URL: &str = "ws://ops.koreainvestment.com:21000";
pub const MOCK_WEBSOCKET_BASE_URL: &str = "ws://ops.koreainvestment.com:31000";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Environment {
    Real,
    Mock,
}

impl Environment {
    pub const fn rest_base_url(self) -> &'static str {
        match self {
            Self::Real => REAL_REST_BASE_URL,
            Self::Mock => MOCK_REST_BASE_URL,
        }
    }

    pub const fn websocket_base_url(self) -> &'static str {
        match self {
            Self::Real => REAL_WEBSOCKET_BASE_URL,
            Self::Mock => MOCK_WEBSOCKET_BASE_URL,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(***)")
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credentials {
    pub app_key: SecretString,
    pub app_secret: SecretString,
}

impl Credentials {
    pub fn new(app_key: impl Into<String>, app_secret: impl Into<String>) -> Result<Self> {
        let app_key = SecretString::new(app_key);
        let app_secret = SecretString::new(app_secret);

        if app_key.is_empty() {
            return Err(Error::config("app key is empty"));
        }

        if app_secret.is_empty() {
            return Err(Error::config("app secret is empty"));
        }

        Ok(Self {
            app_key,
            app_secret,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct AccountNumber(String);

impl AccountNumber {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();

        if value.is_empty() {
            return Err(Error::config("account number is empty"));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AccountNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccountNumber")
            .field(&mask_tail(&self.0, 2))
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductCode(String);

impl ProductCode {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();

        if value.is_empty() {
            return Err(Error::config("account product code is empty"));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Account {
    pub number: AccountNumber,
    pub product_code: ProductCode,
}

impl Account {
    pub fn new(number: AccountNumber, product_code: ProductCode) -> Self {
        Self {
            number,
            product_code,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub environment: Environment,
    pub credentials: Credentials,
    pub account: Option<Account>,
    pub real_ordering_enabled: bool,
}

impl Config {
    pub fn new(environment: Environment, credentials: Credentials) -> Self {
        Self {
            environment,
            credentials,
            account: None,
            real_ordering_enabled: false,
        }
    }

    pub fn with_account(mut self, account: Account) -> Self {
        self.account = Some(account);
        self
    }

    pub fn with_real_ordering_enabled(mut self, enabled: bool) -> Self {
        self.real_ordering_enabled = enabled;
        self
    }

    pub fn rest_base_url(&self) -> &'static str {
        self.environment.rest_base_url()
    }

    pub fn websocket_base_url(&self) -> &'static str {
        self.environment.websocket_base_url()
    }

    pub fn require_account(&self) -> Result<&Account> {
        self.account
            .as_ref()
            .ok_or_else(|| Error::config("account is required"))
    }

    pub fn require_ordering_allowed(&self) -> Result<()> {
        if self.environment == Environment::Real && !self.real_ordering_enabled {
            return Err(Error::config("real ordering requires explicit opt-in"));
        }

        Ok(())
    }
}

fn mask_tail(value: &str, visible_tail_len: usize) -> String {
    let char_count = value.chars().count();

    if char_count <= visible_tail_len {
        return "*".repeat(char_count);
    }

    let masked_len = char_count - visible_tail_len;
    let tail = value.chars().skip(masked_len).collect::<String>();

    format!("{}{}", "*".repeat(masked_len), tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_selects_rest_base_url() {
        assert_eq!(
            Environment::Real.rest_base_url(),
            "https://openapi.koreainvestment.com:9443"
        );
        assert_eq!(
            Environment::Mock.rest_base_url(),
            "https://openapivts.koreainvestment.com:29443"
        );
    }

    #[test]
    fn environment_selects_websocket_base_url() {
        assert_eq!(
            Environment::Real.websocket_base_url(),
            "wss://ops.koreainvestment.com:21000"
        );
        assert_eq!(
            Environment::Mock.websocket_base_url(),
            "wss://vops.koreainvestment.com:31000"
        );
    }

    #[test]
    fn credentials_debug_does_not_expose_secrets() {
        let credentials = Credentials::new("app-key-value", "app-secret-value").unwrap();

        let debug = format!("{credentials:?}");

        assert!(!debug.contains("app-key-value"));
        assert!(!debug.contains("app-secret-value"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn account_debug_does_not_expose_full_number() {
        let account = Account::new(
            AccountNumber::new("12345678").unwrap(),
            ProductCode::new("01").unwrap(),
        );

        let debug = format!("{account:?}");

        assert!(!debug.contains("12345678"));
        assert!(debug.contains("******78"));
    }

    #[test]
    fn config_requires_account_when_missing() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);

        assert_eq!(
            config.require_account(),
            Err(Error::config("account is required"))
        );
    }

    #[test]
    fn mock_ordering_is_allowed_by_default() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Mock, credentials);

        assert_eq!(config.require_ordering_allowed(), Ok(()));
    }

    #[test]
    fn real_ordering_requires_explicit_opt_in() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Real, credentials);

        assert_eq!(
            config.require_ordering_allowed(),
            Err(Error::config("real ordering requires explicit opt-in"))
        );
    }

    #[test]
    fn real_ordering_allows_explicit_opt_in() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Real, credentials).with_real_ordering_enabled(true);

        assert_eq!(config.require_ordering_allowed(), Ok(()));
    }
}

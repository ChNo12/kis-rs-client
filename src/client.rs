use crate::auth::{self, ApprovalKeyResponse, TokenResponse};
use crate::config::{Account, AccountNumber, Config, Credentials, Environment, ProductCode};
use crate::error::{Error, Result};
use crate::http::HttpClient;
#[cfg(feature = "reqwest-client")]
use crate::http::ReqwestHttpClient;
use crate::rest::domestic_stock::Service as DomesticStockService;
use crate::rest::overseas_stock::Service as OverseasStockService;

#[derive(Clone, Debug)]
pub struct Client<T> {
    config: Config,
    http_client: T,
}

impl<T> Client<T> {
    pub fn new(config: Config, http_client: T) -> Self {
        Self {
            config,
            http_client,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub(crate) fn http_client(&self) -> &T {
        &self.http_client
    }

    pub fn rest_base_url(&self) -> &'static str {
        self.config.rest_base_url()
    }

    pub fn websocket_base_url(&self) -> &'static str {
        self.config.websocket_base_url()
    }

    pub fn rest_endpoint_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.rest_base_url(), path)
    }

    pub fn domestic_stock(&self) -> DomesticStockService<'_, T> {
        DomesticStockService::new(self)
    }

    pub fn overseas_stock(&self) -> OverseasStockService<'_, T> {
        OverseasStockService::new(self)
    }
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    environment: Environment,
    credentials: Option<Credentials>,
    account: Option<Account>,
    real_ordering_enabled: bool,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            environment: Environment::Virtual,
            credentials: None,
            account: None,
            real_ordering_enabled: false,
        }
    }

    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    pub fn real(self) -> Self {
        self.environment(Environment::Real)
    }

    pub fn virtual_trading(self) -> Self {
        self.environment(Environment::Virtual)
    }

    pub fn credentials(
        mut self,
        app_key: impl Into<String>,
        app_secret: impl Into<String>,
    ) -> Result<Self> {
        self.credentials = Some(Credentials::new(app_key, app_secret)?);
        Ok(self)
    }

    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn account(
        mut self,
        number: impl Into<String>,
        product_code: impl Into<String>,
    ) -> Result<Self> {
        self.account = Some(Account::new(
            AccountNumber::new(number)?,
            ProductCode::new(product_code)?,
        ));
        Ok(self)
    }

    pub fn with_account(mut self, account: Account) -> Self {
        self.account = Some(account);
        self
    }

    pub fn enable_real_ordering(mut self) -> Self {
        self.real_ordering_enabled = true;
        self
    }

    pub fn build_config(self) -> Result<Config> {
        let credentials = self
            .credentials
            .ok_or_else(|| Error::config("credentials are required"))?;
        let mut config = Config::new(self.environment, credentials)
            .with_real_ordering_enabled(self.real_ordering_enabled);

        if let Some(account) = self.account {
            config = config.with_account(account);
        }

        Ok(config)
    }

    pub fn build_with_http<T>(self, http_client: T) -> Result<Client<T>> {
        Ok(Client::new(self.build_config()?, http_client))
    }

    #[cfg(feature = "reqwest-client")]
    pub fn build_reqwest(self) -> Result<Client<ReqwestHttpClient>> {
        self.build_with_http(ReqwestHttpClient::new())
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HttpClient> Client<T> {
    pub async fn issue_token(&self) -> Result<TokenResponse> {
        auth::issue_token(self).await
    }

    pub async fn issue_approval_key(&self) -> Result<ApprovalKeyResponse> {
        auth::issue_approval_key(self).await
    }
}

#[cfg(feature = "reqwest-client")]
impl Client<ReqwestHttpClient> {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn new_reqwest(config: Config) -> Self {
        Self::new(config, ReqwestHttpClient::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Credentials, Environment};

    #[derive(Debug)]
    struct NoopHttpClient;

    #[test]
    fn builds_rest_endpoint_url() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Virtual, credentials);
        let client = Client::new(config, NoopHttpClient);

        assert_eq!(
            client.rest_endpoint_url("/oauth2/tokenP"),
            "https://openapivts.koreainvestment.com:29443/oauth2/tokenP"
        );
    }

    #[test]
    fn exposes_websocket_base_url() {
        let credentials = Credentials::new("app-key", "app-secret").unwrap();
        let config = Config::new(Environment::Virtual, credentials);
        let client = Client::new(config, NoopHttpClient);

        assert_eq!(
            client.websocket_base_url(),
            "ws://ops.koreainvestment.com:31000"
        );
    }

    #[test]
    fn builder_defaults_to_virtual_environment() {
        let client = ClientBuilder::new()
            .credentials("app-key", "app-secret")
            .unwrap()
            .build_with_http(NoopHttpClient)
            .unwrap();

        assert_eq!(client.config().environment, Environment::Virtual);
    }

    #[test]
    fn builder_rejects_missing_credentials() {
        let error = ClientBuilder::new()
            .build_with_http(NoopHttpClient)
            .unwrap_err();

        assert_eq!(error, Error::config("credentials are required"));
    }

    #[test]
    fn builder_keeps_real_ordering_disabled_by_default() {
        let client = ClientBuilder::new()
            .real()
            .credentials("app-key", "app-secret")
            .unwrap()
            .build_with_http(NoopHttpClient)
            .unwrap();

        assert!(!client.config().real_ordering_enabled);
    }

    #[test]
    fn builder_enables_real_ordering_explicitly() {
        let client = ClientBuilder::new()
            .real()
            .enable_real_ordering()
            .credentials("app-key", "app-secret")
            .unwrap()
            .build_with_http(NoopHttpClient)
            .unwrap();

        assert!(client.config().real_ordering_enabled);
    }
}

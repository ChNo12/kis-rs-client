use crate::auth::AccessToken;
use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::rest::domestic_stock::common::{Continuation, Endpoint, get, require_output};
use crate::rest::domestic_stock::market_analysis::Service;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) async fn get_output<T: HttpClient>(
    service: &Service<'_, T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<(Value, Continuation)> {
    let response = get(
        service.client,
        access_token,
        endpoint,
        params,
        None,
        parse_context,
    )
    .await?;

    require_output(response, parse_context)
}

pub(crate) fn parse_typed<T: DeserializeOwned>(
    value: Value,
    parse_context: &'static str,
) -> Result<T> {
    serde_json::from_value(value)
        .map_err(|error| Error::parse(format!("failed to parse {parse_context}: {error}")))
}

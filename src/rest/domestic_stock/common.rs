use serde::Deserialize;
use serde_json::{Map, Value};

use crate::auth::{AUTHORIZATION_HEADER, AccessToken};
use crate::client::Client;
use crate::error::{Error, Result};
use crate::http::{HttpClient, Request, Response};
use crate::rest::{ApiResponseStatus, CONTENT_TYPE_JSON};

pub(crate) const APP_KEY_HEADER: &str = "appkey";
pub(crate) const APP_SECRET_HEADER: &str = "appsecret";
pub(crate) const TR_ID_HEADER: &str = "tr_id";
pub(crate) const CUSTTYPE_HEADER: &str = "custtype";
pub(crate) const TR_CONT_HEADER: &str = "tr_cont";
pub(crate) const PERSONAL_CUSTOMER_TYPE: &str = "P";
pub(crate) const FID_COND_MRKT_DIV_CODE: &str = "FID_COND_MRKT_DIV_CODE";
pub(crate) const FID_INPUT_ISCD: &str = "FID_INPUT_ISCD";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketDivision {
    Krx,
    Nxt,
    Unified,
}

impl MarketDivision {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Krx => "J",
            Self::Nxt => "NX",
            Self::Unified => "UN",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockCode(String);

impl StockCode {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();

        if value.is_empty() {
            return Err(Error::config("stock code is empty"));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Continuation {
    pub tr_cont: Option<String>,
    pub ctx_area_fk: Option<String>,
    pub ctx_area_nk: Option<String>,
}

impl Continuation {
    pub fn from_response(response: &Response) -> Self {
        Self {
            tr_cont: optional_header(response, TR_CONT_HEADER),
            ctx_area_fk: optional_header(response, "ctx_area_fk"),
            ctx_area_nk: optional_header(response, "ctx_area_nk"),
        }
    }

    pub fn has_next(&self) -> bool {
        matches!(self.tr_cont.as_deref(), Some("M" | "F"))
    }

    pub fn next_request(&self) -> Option<Self> {
        self.has_next().then(|| Self {
            tr_cont: Some("N".to_string()),
            ctx_area_fk: self.ctx_area_fk.clone(),
            ctx_area_nk: self.ctx_area_nk.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Endpoint {
    pub path: &'static str,
    pub tr_id: &'static str,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawApiResponse {
    pub output: Option<Value>,
    pub output1: Option<Value>,
    pub output2: Option<Value>,
    pub output3: Option<Value>,
    pub output4: Option<Value>,
    pub ctx_area_fk: Option<String>,
    pub ctx_area_nk: Option<String>,
    pub ctx_area_fk100: Option<String>,
    pub ctx_area_nk100: Option<String>,
    pub ctx_area_fk200: Option<String>,
    pub ctx_area_nk200: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RawResponse {
    pub output: Option<Value>,
    pub output1: Option<Value>,
    pub output2: Option<Value>,
    pub output3: Option<Value>,
    pub output4: Option<Value>,
    pub continuation: Continuation,
}

pub(crate) async fn get<T: HttpClient>(
    client: &Client<T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    query_params: Vec<(&'static str, String)>,
    continuation: Option<&Continuation>,
    parse_context: &'static str,
) -> Result<RawResponse> {
    let request = build_get_request(client, access_token, endpoint, query_params, continuation);
    let response = client.http_client().send(request).await?;

    parse_response(response, parse_context)
}

pub(crate) async fn post<T: HttpClient>(
    client: &Client<T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    body_params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<RawResponse> {
    let request = build_post_request(client, access_token, endpoint, body_params, parse_context)?;
    let response = client.http_client().send(request).await?;

    parse_response(response, parse_context)
}

pub(crate) fn build_get_request<T>(
    client: &Client<T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    query_params: Vec<(&'static str, String)>,
    continuation: Option<&Continuation>,
) -> Request {
    let mut request = Request::get(client.rest_endpoint_url(endpoint.path))
        .with_header("content-type", CONTENT_TYPE_JSON)
        .with_header(AUTHORIZATION_HEADER, access_token.bearer_authorization())
        .with_header(
            APP_KEY_HEADER,
            client.config().credentials.app_key.expose_secret(),
        )
        .with_header(
            APP_SECRET_HEADER,
            client.config().credentials.app_secret.expose_secret(),
        )
        .with_header(TR_ID_HEADER, endpoint.tr_id)
        .with_header(CUSTTYPE_HEADER, PERSONAL_CUSTOMER_TYPE)
        .with_header(
            TR_CONT_HEADER,
            continuation
                .and_then(|continuation| continuation.tr_cont.as_deref())
                .unwrap_or(""),
        );

    for (name, value) in query_params {
        request = request.with_query_param(name, value);
    }

    request
}

pub(crate) fn build_post_request<T>(
    client: &Client<T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    body_params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<Request> {
    let body = serialize_body_params(body_params, parse_context)?;

    Ok(Request::post(client.rest_endpoint_url(endpoint.path))
        .with_header("content-type", CONTENT_TYPE_JSON)
        .with_header(AUTHORIZATION_HEADER, access_token.bearer_authorization())
        .with_header(
            APP_KEY_HEADER,
            client.config().credentials.app_key.expose_secret(),
        )
        .with_header(
            APP_SECRET_HEADER,
            client.config().credentials.app_secret.expose_secret(),
        )
        .with_header(TR_ID_HEADER, endpoint.tr_id)
        .with_header(CUSTTYPE_HEADER, PERSONAL_CUSTOMER_TYPE)
        .with_body(body))
}

pub(crate) fn parse_response(
    response: Response,
    parse_context: &'static str,
) -> Result<RawResponse> {
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

    let mut continuation = Continuation::from_response(&response);
    let body = serde_json::from_slice::<RawApiResponse>(response.body()).map_err(|error| {
        Error::parse(format!("failed to parse {parse_context} response: {error}"))
    })?;

    if continuation.ctx_area_fk.is_none() {
        continuation.ctx_area_fk = body
            .ctx_area_fk
            .clone()
            .or(body.ctx_area_fk100.clone())
            .or(body.ctx_area_fk200.clone());
    }

    if continuation.ctx_area_nk.is_none() {
        continuation.ctx_area_nk = body
            .ctx_area_nk
            .clone()
            .or(body.ctx_area_nk100.clone())
            .or(body.ctx_area_nk200.clone());
    }

    Ok(RawResponse {
        output: body.output,
        output1: body.output1,
        output2: body.output2,
        output3: body.output3,
        output4: body.output4,
        continuation,
    })
}

pub(crate) fn require_output(
    response: RawResponse,
    parse_context: &'static str,
) -> Result<(Value, Continuation)> {
    let output = response
        .output
        .ok_or_else(|| Error::parse(format!("{parse_context} response missing output")))?;

    Ok((output, response.continuation))
}

pub(crate) fn require_output_pair(
    response: RawResponse,
    parse_context: &'static str,
) -> Result<(Value, Value, Continuation)> {
    let output1 = response
        .output1
        .ok_or_else(|| Error::parse(format!("{parse_context} response missing output1")))?;
    let output2 = response
        .output2
        .ok_or_else(|| Error::parse(format!("{parse_context} response missing output2")))?;

    Ok((output1, output2, response.continuation))
}

fn optional_header(response: &Response, name: &str) -> Option<String> {
    response
        .header(name)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn status_fallback_message(response: &Response) -> String {
    if response.is_success() {
        "KIS API request failed".to_string()
    } else {
        format!("unexpected HTTP status: {}", response.status_code())
    }
}

fn serialize_body_params(
    body_params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<Vec<u8>> {
    let body = body_params
        .into_iter()
        .map(|(name, value)| (name.to_string(), Value::String(value)))
        .collect::<Map<_, _>>();

    serde_json::to_vec(&body).map_err(|error| {
        Error::parse(format!(
            "failed to serialize {parse_context} request: {error}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Header;

    #[test]
    fn market_division_uses_kis_codes() {
        assert_eq!(MarketDivision::Krx.as_str(), "J");
        assert_eq!(MarketDivision::Nxt.as_str(), "NX");
        assert_eq!(MarketDivision::Unified.as_str(), "UN");
    }

    #[test]
    fn stock_code_rejects_empty_value() {
        let error = StockCode::new("").unwrap_err();

        assert_eq!(error, Error::config("stock code is empty"));
    }

    #[test]
    fn continuation_reads_response_headers() {
        let response = Response::new(200, "{}").with_headers([
            Header::new("tr_cont", "M"),
            Header::new("ctx_area_fk", "fk"),
            Header::new("ctx_area_nk", "nk"),
        ]);

        let continuation = Continuation::from_response(&response);

        assert!(continuation.has_next());
        assert_eq!(continuation.tr_cont.as_deref(), Some("M"));
        assert_eq!(continuation.ctx_area_fk.as_deref(), Some("fk"));
        assert_eq!(continuation.ctx_area_nk.as_deref(), Some("nk"));
    }

    #[test]
    fn continuation_builds_next_request_marker() {
        let continuation = Continuation {
            tr_cont: Some("M".to_string()),
            ctx_area_fk: Some("fk".to_string()),
            ctx_area_nk: Some("nk".to_string()),
        };

        let next = continuation.next_request().unwrap();

        assert_eq!(next.tr_cont.as_deref(), Some("N"));
        assert_eq!(next.ctx_area_fk.as_deref(), Some("fk"));
        assert_eq!(next.ctx_area_nk.as_deref(), Some("nk"));
    }
}

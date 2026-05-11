use super::{DoubleOutputResponse, Service, SingleOutputResponse};
use crate::auth::AccessToken;
use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::rest::domestic_stock::common::{
    Endpoint, FID_COND_MRKT_DIV_CODE, FID_INPUT_ISCD, MarketDivision, StockCode, get,
    require_output, require_output_pair,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) const FID_PERIOD_DIV_CODE: &str = "FID_PERIOD_DIV_CODE";
pub(crate) const FID_ORG_ADJ_PRC: &str = "FID_ORG_ADJ_PRC";
pub(crate) const FID_INPUT_DATE_1: &str = "FID_INPUT_DATE_1";
pub(crate) const FID_INPUT_DATE_2: &str = "FID_INPUT_DATE_2";
pub(crate) const FID_INPUT_HOUR_1: &str = "FID_INPUT_HOUR_1";
pub(crate) const FID_PW_DATA_INCU_YN: &str = "FID_PW_DATA_INCU_YN";
pub(crate) const FID_ETC_CLS_CODE: &str = "FID_ETC_CLS_CODE";
pub(crate) const FID_FAKE_TICK_INCU_YN: &str = "FID_FAKE_TICK_INCU_YN";

pub(crate) async fn get_output<T: HttpClient>(
    service: &Service<'_, T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<SingleOutputResponse> {
    let response = get(
        service.client,
        access_token,
        endpoint,
        params,
        None,
        parse_context,
    )
    .await?;
    let (output, continuation) = require_output(response, parse_context)?;

    Ok(SingleOutputResponse {
        output,
        continuation,
    })
}

pub(crate) async fn get_output_pair<T: HttpClient>(
    service: &Service<'_, T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<DoubleOutputResponse> {
    let response = get(
        service.client,
        access_token,
        endpoint,
        params,
        None,
        parse_context,
    )
    .await?;
    let (output1, output2, continuation) = require_output_pair(response, parse_context)?;

    Ok(DoubleOutputResponse {
        output1,
        output2,
        continuation,
    })
}

pub(crate) fn simple_stock_params(
    market_division: MarketDivision,
    stock_code: &StockCode,
) -> Vec<(&'static str, String)> {
    vec![
        (FID_COND_MRKT_DIV_CODE, market_division.as_str().to_string()),
        (FID_INPUT_ISCD, stock_code.as_str().to_string()),
    ]
}

pub(crate) fn parse_typed<T: DeserializeOwned>(
    value: Value,
    parse_context: &'static str,
) -> Result<T> {
    serde_json::from_value(value)
        .map_err(|error| Error::parse(format!("failed to parse {parse_context}: {error}")))
}

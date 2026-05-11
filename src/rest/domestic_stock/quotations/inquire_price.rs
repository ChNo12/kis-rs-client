use serde_json::Value;

use super::Service;
use super::common::{parse_typed, simple_stock_params};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::quotations::InquirePriceOutput;
use crate::rest::domestic_stock::common::{
    Continuation, Endpoint, MarketDivision, StockCode, get, require_output,
};

pub const INQUIRE_PRICE_PATH: &str = "/uapi/domestic-stock/v1/quotations/inquire-price";
pub const INQUIRE_PRICE_TR_ID: &str = "FHKST01010100";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquirePriceRequest {
    pub market_division: MarketDivision,
    pub stock_code: StockCode,
}

impl InquirePriceRequest {
    pub fn new(market_division: MarketDivision, stock_code: StockCode) -> Self {
        Self {
            market_division,
            stock_code,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquirePriceResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl InquirePriceResponse {
    pub fn current_price(&self) -> Option<&str> {
        self.output.get("stck_prpr").and_then(Value::as_str)
    }

    pub fn typed(&self) -> Result<InquirePriceOutput> {
        parse_typed(self.output.clone(), "domestic stock price output")
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_price(
        &self,
        access_token: &AccessToken,
        request: InquirePriceRequest,
    ) -> Result<InquirePriceResponse> {
        let response = get(
            self.client,
            access_token,
            Endpoint {
                path: INQUIRE_PRICE_PATH,
                tr_id: INQUIRE_PRICE_TR_ID,
            },
            simple_stock_params(request.market_division, &request.stock_code),
            None,
            "domestic stock price",
        )
        .await?;
        let (output, continuation) = require_output(response, "domestic stock price")?;

        Ok(InquirePriceResponse {
            output,
            continuation,
        })
    }
}

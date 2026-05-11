use super::Service;
use super::common::{
    FID_ORG_ADJ_PRC, FID_PERIOD_DIV_CODE, get_output, parse_typed, simple_stock_params,
};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::quotations::InquireDailyPriceItem;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::{Endpoint, MarketDivision, StockCode};
use serde_json::Value;

pub const INQUIRE_DAILY_PRICE_PATH: &str = "/uapi/domestic-stock/v1/quotations/inquire-daily-price";
pub const INQUIRE_DAILY_PRICE_TR_ID: &str = "FHKST01010400";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireDailyPriceRequest {
    pub market_division: MarketDivision,
    pub stock_code: StockCode,
    pub period_division_code: String,
    pub adjusted_price_code: String,
}

impl InquireDailyPriceRequest {
    pub fn new(market_division: MarketDivision, stock_code: StockCode) -> Self {
        Self {
            market_division,
            stock_code,
            period_division_code: "D".to_string(),
            adjusted_price_code: "1".to_string(),
        }
    }

    pub fn with_period_division_code(mut self, value: impl Into<String>) -> Self {
        self.period_division_code = value.into();
        self
    }

    pub fn with_adjusted_price_code(mut self, value: impl Into<String>) -> Self {
        self.adjusted_price_code = value.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireDailyPriceResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl InquireDailyPriceResponse {
    pub fn typed(&self) -> Result<Vec<InquireDailyPriceItem>> {
        parse_typed(self.output.clone(), "domestic stock daily price output")
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_daily_price(
        &self,
        access_token: &AccessToken,
        request: InquireDailyPriceRequest,
    ) -> Result<InquireDailyPriceResponse> {
        let mut params = simple_stock_params(request.market_division, &request.stock_code);
        params.push((FID_PERIOD_DIV_CODE, request.period_division_code));
        params.push((FID_ORG_ADJ_PRC, request.adjusted_price_code));

        let response = get_output(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_DAILY_PRICE_PATH,
                tr_id: INQUIRE_DAILY_PRICE_TR_ID,
            },
            params,
            "domestic stock daily price",
        )
        .await?;

        Ok(InquireDailyPriceResponse {
            output: response.output,
            continuation: response.continuation,
        })
    }
}

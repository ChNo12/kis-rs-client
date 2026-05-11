use super::Service;
use super::common::{get_output, parse_typed, simple_stock_params};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::quotations::InquireCcnlItem;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::{Endpoint, MarketDivision, StockCode};
use serde_json::Value;

pub const INQUIRE_CCNL_PATH: &str = "/uapi/domestic-stock/v1/quotations/inquire-ccnl";
pub const INQUIRE_CCNL_TR_ID: &str = "FHKST01010300";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireCcnlRequest {
    pub market_division: MarketDivision,
    pub stock_code: StockCode,
}

impl InquireCcnlRequest {
    pub fn new(market_division: MarketDivision, stock_code: StockCode) -> Self {
        Self {
            market_division,
            stock_code,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireCcnlResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl InquireCcnlResponse {
    pub fn typed(&self) -> Result<Vec<InquireCcnlItem>> {
        parse_typed(self.output.clone(), "domestic stock conclusion output")
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_ccnl(
        &self,
        access_token: &AccessToken,
        request: InquireCcnlRequest,
    ) -> Result<InquireCcnlResponse> {
        let response = get_output(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_CCNL_PATH,
                tr_id: INQUIRE_CCNL_TR_ID,
            },
            simple_stock_params(request.market_division, &request.stock_code),
            "domestic stock conclusion",
        )
        .await?;

        Ok(InquireCcnlResponse {
            output: response.output,
            continuation: response.continuation,
        })
    }
}

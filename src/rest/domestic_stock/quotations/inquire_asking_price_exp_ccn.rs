use super::Service;
use super::common::{get_output_pair, parse_typed, simple_stock_params};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::quotations::{
    InquireAskingPriceExpCcnOutput, InquireAskingPriceExpCcnOutput1,
    InquireAskingPriceExpCcnOutput2,
};
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::{Endpoint, MarketDivision, StockCode};
use serde_json::Value;

pub const INQUIRE_ASKING_PRICE_EXP_CCN_PATH: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-asking-price-exp-ccn";
pub const INQUIRE_ASKING_PRICE_EXP_CCN_TR_ID: &str = "FHKST01010200";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireAskingPriceExpCcnRequest {
    pub market_division: MarketDivision,
    pub stock_code: StockCode,
}

impl InquireAskingPriceExpCcnRequest {
    pub fn new(market_division: MarketDivision, stock_code: StockCode) -> Self {
        Self {
            market_division,
            stock_code,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireAskingPriceExpCcnResponse {
    pub output1: Value,
    pub output2: Value,
    pub continuation: Continuation,
}

impl InquireAskingPriceExpCcnResponse {
    pub fn typed(&self) -> Result<InquireAskingPriceExpCcnOutput> {
        Ok(InquireAskingPriceExpCcnOutput {
            asking_price: self.typed_output1()?,
            expected_conclusion: self.typed_output2()?,
        })
    }

    pub fn typed_output1(&self) -> Result<InquireAskingPriceExpCcnOutput1> {
        parse_typed(
            self.output1.clone(),
            "domestic stock asking price expected conclusion output1",
        )
    }

    pub fn typed_output2(&self) -> Result<InquireAskingPriceExpCcnOutput2> {
        parse_typed(
            self.output2.clone(),
            "domestic stock asking price expected conclusion output2",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_asking_price_exp_ccn(
        &self,
        access_token: &AccessToken,
        request: InquireAskingPriceExpCcnRequest,
    ) -> Result<InquireAskingPriceExpCcnResponse> {
        let response = get_output_pair(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_ASKING_PRICE_EXP_CCN_PATH,
                tr_id: INQUIRE_ASKING_PRICE_EXP_CCN_TR_ID,
            },
            simple_stock_params(request.market_division, &request.stock_code),
            "domestic stock asking price expected conclusion",
        )
        .await?;

        Ok(InquireAskingPriceExpCcnResponse {
            output1: response.output1,
            output2: response.output2,
            continuation: response.continuation,
        })
    }
}

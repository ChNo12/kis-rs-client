use super::Service;
use super::common::{
    FID_INPUT_DATE_1, FID_INPUT_DATE_2, FID_ORG_ADJ_PRC, FID_PERIOD_DIV_CODE, get_output_pair,
    parse_typed, simple_stock_params,
};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::quotations::{
    InquireDailyItemChartPriceItem, InquireDailyItemChartPriceOutput,
    InquireDailyItemChartPriceSummary,
};
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::{Endpoint, MarketDivision, StockCode};
use serde_json::Value;

pub const INQUIRE_DAILY_ITEM_CHART_PRICE_PATH: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice";
pub const INQUIRE_DAILY_ITEM_CHART_PRICE_TR_ID: &str = "FHKST03010100";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireDailyItemChartPriceRequest {
    pub market_division: MarketDivision,
    pub stock_code: StockCode,
    pub start_date: String,
    pub end_date: String,
    pub period_division_code: String,
    pub adjusted_price_code: String,
}

impl InquireDailyItemChartPriceRequest {
    pub fn new(
        market_division: MarketDivision,
        stock_code: StockCode,
        start_date: impl Into<String>,
        end_date: impl Into<String>,
    ) -> Self {
        Self {
            market_division,
            stock_code,
            start_date: start_date.into(),
            end_date: end_date.into(),
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
pub struct InquireDailyItemChartPriceResponse {
    pub output1: Value,
    pub output2: Value,
    pub continuation: Continuation,
}

impl InquireDailyItemChartPriceResponse {
    pub fn typed(&self) -> Result<InquireDailyItemChartPriceOutput> {
        Ok(InquireDailyItemChartPriceOutput {
            summary: self.typed_output1()?,
            prices: self.typed_output2()?,
        })
    }

    pub fn typed_output1(&self) -> Result<InquireDailyItemChartPriceSummary> {
        parse_typed(
            self.output1.clone(),
            "domestic stock daily item chart price output1",
        )
    }

    pub fn typed_output2(&self) -> Result<Vec<InquireDailyItemChartPriceItem>> {
        parse_typed(
            self.output2.clone(),
            "domestic stock daily item chart price output2",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_daily_item_chart_price(
        &self,
        access_token: &AccessToken,
        request: InquireDailyItemChartPriceRequest,
    ) -> Result<InquireDailyItemChartPriceResponse> {
        let mut params = simple_stock_params(request.market_division, &request.stock_code);
        params.push((FID_INPUT_DATE_1, request.start_date));
        params.push((FID_INPUT_DATE_2, request.end_date));
        params.push((FID_PERIOD_DIV_CODE, request.period_division_code));
        params.push((FID_ORG_ADJ_PRC, request.adjusted_price_code));

        let response = get_output_pair(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_DAILY_ITEM_CHART_PRICE_PATH,
                tr_id: INQUIRE_DAILY_ITEM_CHART_PRICE_TR_ID,
            },
            params,
            "domestic stock daily item chart price",
        )
        .await?;

        Ok(InquireDailyItemChartPriceResponse {
            output1: response.output1,
            output2: response.output2,
            continuation: response.continuation,
        })
    }
}

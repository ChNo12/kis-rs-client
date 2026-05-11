use super::Service;
use super::common::{
    FID_ETC_CLS_CODE, FID_INPUT_HOUR_1, FID_PW_DATA_INCU_YN, get_output_pair, parse_typed,
    simple_stock_params,
};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::quotations::{
    InquireTimeItemChartPriceItem, InquireTimeItemChartPriceOutput,
    InquireTimeItemChartPriceSummary,
};
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::{Endpoint, MarketDivision, StockCode};
use serde_json::Value;

pub const INQUIRE_TIME_ITEM_CHART_PRICE_PATH: &str =
    "/uapi/domestic-stock/v1/quotations/inquire-time-itemchartprice";
pub const INQUIRE_TIME_ITEM_CHART_PRICE_TR_ID: &str = "FHKST03010200";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireTimeItemChartPriceRequest {
    pub market_division: MarketDivision,
    pub stock_code: StockCode,
    pub input_hour: String,
    pub include_past_data: bool,
    pub etc_class_code: String,
}

impl InquireTimeItemChartPriceRequest {
    pub fn new(
        market_division: MarketDivision,
        stock_code: StockCode,
        input_hour: impl Into<String>,
    ) -> Self {
        Self {
            market_division,
            stock_code,
            input_hour: input_hour.into(),
            include_past_data: false,
            etc_class_code: String::new(),
        }
    }

    pub fn include_past_data(mut self, value: bool) -> Self {
        self.include_past_data = value;
        self
    }

    pub fn with_etc_class_code(mut self, value: impl Into<String>) -> Self {
        self.etc_class_code = value.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireTimeItemChartPriceResponse {
    pub output1: Value,
    pub output2: Value,
    pub continuation: Continuation,
}

impl InquireTimeItemChartPriceResponse {
    pub fn typed(&self) -> Result<InquireTimeItemChartPriceOutput> {
        Ok(InquireTimeItemChartPriceOutput {
            summary: self.typed_output1()?,
            items: self.typed_output2()?,
        })
    }

    pub fn typed_output1(&self) -> Result<InquireTimeItemChartPriceSummary> {
        parse_typed(
            self.output1.clone(),
            "domestic stock time item chart price output1",
        )
    }

    pub fn typed_output2(&self) -> Result<Vec<InquireTimeItemChartPriceItem>> {
        parse_typed(
            self.output2.clone(),
            "domestic stock time item chart price output2",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_time_item_chart_price(
        &self,
        access_token: &AccessToken,
        request: InquireTimeItemChartPriceRequest,
    ) -> Result<InquireTimeItemChartPriceResponse> {
        let mut params = simple_stock_params(request.market_division, &request.stock_code);
        params.push((FID_INPUT_HOUR_1, request.input_hour));
        params.push((
            FID_PW_DATA_INCU_YN,
            if request.include_past_data { "Y" } else { "N" }.to_string(),
        ));
        params.push((FID_ETC_CLS_CODE, request.etc_class_code));

        let response = get_output_pair(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_TIME_ITEM_CHART_PRICE_PATH,
                tr_id: INQUIRE_TIME_ITEM_CHART_PRICE_TR_ID,
            },
            params,
            "domestic stock time item chart price",
        )
        .await?;

        Ok(InquireTimeItemChartPriceResponse {
            output1: response.output1,
            output2: response.output2,
            continuation: response.continuation,
        })
    }
}

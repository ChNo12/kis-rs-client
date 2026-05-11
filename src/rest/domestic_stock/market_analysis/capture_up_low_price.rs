use super::Service;
use super::common::{get_output, parse_typed};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::market_analysis::CaptureUpLowPriceItem;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use serde_json::Value;

pub const CAPTURE_UP_LOW_PRICE_PATH: &str = "/uapi/domestic-stock/v1/quotations/capture-uplowprice";
pub const CAPTURE_UP_LOW_PRICE_TR_ID: &str = "FHKST130000C0";

const FID_COND_MRKT_DIV_CODE: &str = "FID_COND_MRKT_DIV_CODE";
const FID_COND_SCR_DIV_CODE: &str = "FID_COND_SCR_DIV_CODE";
const FID_PRC_CLS_CODE: &str = "FID_PRC_CLS_CODE";
const FID_DIV_CLS_CODE: &str = "FID_DIV_CLS_CODE";
const FID_INPUT_ISCD: &str = "FID_INPUT_ISCD";
const FID_TRGT_CLS_CODE: &str = "FID_TRGT_CLS_CODE";
const FID_TRGT_EXLS_CLS_CODE: &str = "FID_TRGT_EXLS_CLS_CODE";
const FID_INPUT_PRICE_1: &str = "FID_INPUT_PRICE_1";
const FID_INPUT_PRICE_2: &str = "FID_INPUT_PRICE_2";
const FID_VOL_CNT: &str = "FID_VOL_CNT";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureUpLowPriceRequest {
    pub market_division_code: String,
    pub screen_division_code: String,
    pub price_class_code: String,
    pub division_class_code: String,
    pub input_code: String,
    pub target_class_code: String,
    pub target_exclusion_class_code: String,
    pub input_price1: String,
    pub input_price2: String,
    pub volume_count: String,
}

impl CaptureUpLowPriceRequest {
    pub fn new() -> Self {
        Self {
            market_division_code: "J".to_string(),
            screen_division_code: "11300".to_string(),
            price_class_code: "0".to_string(),
            division_class_code: "0".to_string(),
            input_code: "0000".to_string(),
            target_class_code: String::new(),
            target_exclusion_class_code: String::new(),
            input_price1: String::new(),
            input_price2: String::new(),
            volume_count: String::new(),
        }
    }

    pub fn with_market_division_code(mut self, value: impl Into<String>) -> Self {
        self.market_division_code = value.into();
        self
    }

    pub fn with_price_class_code(mut self, value: impl Into<String>) -> Self {
        self.price_class_code = value.into();
        self
    }

    pub fn with_division_class_code(mut self, value: impl Into<String>) -> Self {
        self.division_class_code = value.into();
        self
    }

    pub fn with_input_code(mut self, value: impl Into<String>) -> Self {
        self.input_code = value.into();
        self
    }

    pub fn with_target_class_code(mut self, value: impl Into<String>) -> Self {
        self.target_class_code = value.into();
        self
    }

    pub fn with_target_exclusion_class_code(mut self, value: impl Into<String>) -> Self {
        self.target_exclusion_class_code = value.into();
        self
    }

    pub fn with_price_range(
        mut self,
        lower_bound: impl Into<String>,
        upper_bound: impl Into<String>,
    ) -> Self {
        self.input_price1 = lower_bound.into();
        self.input_price2 = upper_bound.into();
        self
    }

    pub fn with_volume_count(mut self, value: impl Into<String>) -> Self {
        self.volume_count = value.into();
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        vec![
            (FID_COND_MRKT_DIV_CODE, self.market_division_code.clone()),
            (FID_COND_SCR_DIV_CODE, self.screen_division_code.clone()),
            (FID_PRC_CLS_CODE, self.price_class_code.clone()),
            (FID_DIV_CLS_CODE, self.division_class_code.clone()),
            (FID_INPUT_ISCD, self.input_code.clone()),
            (FID_TRGT_CLS_CODE, self.target_class_code.clone()),
            (
                FID_TRGT_EXLS_CLS_CODE,
                self.target_exclusion_class_code.clone(),
            ),
            (FID_INPUT_PRICE_1, self.input_price1.clone()),
            (FID_INPUT_PRICE_2, self.input_price2.clone()),
            (FID_VOL_CNT, self.volume_count.clone()),
        ]
    }
}

impl Default for CaptureUpLowPriceRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureUpLowPriceResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl CaptureUpLowPriceResponse {
    pub fn typed(&self) -> Result<Vec<CaptureUpLowPriceItem>> {
        parse_typed(
            self.output.clone(),
            "domestic stock capture up low price output",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn capture_up_low_price(
        &self,
        access_token: &AccessToken,
        request: CaptureUpLowPriceRequest,
    ) -> Result<CaptureUpLowPriceResponse> {
        let (output, continuation) = get_output(
            self,
            access_token,
            Endpoint {
                path: CAPTURE_UP_LOW_PRICE_PATH,
                tr_id: CAPTURE_UP_LOW_PRICE_TR_ID,
            },
            request.params(),
            "domestic stock capture up low price",
        )
        .await?;

        Ok(CaptureUpLowPriceResponse {
            output,
            continuation,
        })
    }
}

use super::Service;
use super::common::{get_output, parse_typed};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::ranking::AfterHourBalanceItem;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use serde_json::Value;

pub const AFTER_HOUR_BALANCE_PATH: &str = "/uapi/domestic-stock/v1/ranking/after-hour-balance";
pub const AFTER_HOUR_BALANCE_TR_ID: &str = "FHPST01760000";

const FID_INPUT_PRICE_1: &str = "fid_input_price_1";
const FID_COND_MRKT_DIV_CODE: &str = "fid_cond_mrkt_div_code";
const FID_COND_SCR_DIV_CODE: &str = "fid_cond_scr_div_code";
const FID_RANK_SORT_CLS_CODE: &str = "fid_rank_sort_cls_code";
const FID_DIV_CLS_CODE: &str = "fid_div_cls_code";
const FID_INPUT_ISCD: &str = "fid_input_iscd";
const FID_TRGT_EXLS_CLS_CODE: &str = "fid_trgt_exls_cls_code";
const FID_TRGT_CLS_CODE: &str = "fid_trgt_cls_code";
const FID_VOL_CNT: &str = "fid_vol_cnt";
const FID_INPUT_PRICE_2: &str = "fid_input_price_2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterHourBalanceRequest {
    pub input_price1: String,
    pub market_division_code: String,
    pub screen_division_code: String,
    pub rank_sort_class_code: String,
    pub division_class_code: String,
    pub input_code: String,
    pub target_exclusion_class_code: String,
    pub target_class_code: String,
    pub volume_count: String,
    pub input_price2: String,
    pub continuation: Option<Continuation>,
}

impl AfterHourBalanceRequest {
    pub fn new() -> Self {
        Self {
            input_price1: String::new(),
            market_division_code: "J".to_string(),
            screen_division_code: "20176".to_string(),
            rank_sort_class_code: "1".to_string(),
            division_class_code: "0".to_string(),
            input_code: "0000".to_string(),
            target_exclusion_class_code: "0".to_string(),
            target_class_code: "0".to_string(),
            volume_count: String::new(),
            input_price2: String::new(),
            continuation: None,
        }
    }

    pub fn with_market_division_code(mut self, value: impl Into<String>) -> Self {
        self.market_division_code = value.into();
        self
    }

    pub fn with_rank_sort_class_code(mut self, value: impl Into<String>) -> Self {
        self.rank_sort_class_code = value.into();
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

    pub fn with_target_exclusion_class_code(mut self, value: impl Into<String>) -> Self {
        self.target_exclusion_class_code = value.into();
        self
    }

    pub fn with_target_class_code(mut self, value: impl Into<String>) -> Self {
        self.target_class_code = value.into();
        self
    }

    pub fn with_volume_count(mut self, value: impl Into<String>) -> Self {
        self.volume_count = value.into();
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

    pub fn with_continuation(mut self, value: Continuation) -> Self {
        self.continuation = Some(value);
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        vec![
            (FID_INPUT_PRICE_1, self.input_price1.clone()),
            (FID_COND_MRKT_DIV_CODE, self.market_division_code.clone()),
            (FID_COND_SCR_DIV_CODE, self.screen_division_code.clone()),
            (FID_RANK_SORT_CLS_CODE, self.rank_sort_class_code.clone()),
            (FID_DIV_CLS_CODE, self.division_class_code.clone()),
            (FID_INPUT_ISCD, self.input_code.clone()),
            (
                FID_TRGT_EXLS_CLS_CODE,
                self.target_exclusion_class_code.clone(),
            ),
            (FID_TRGT_CLS_CODE, self.target_class_code.clone()),
            (FID_VOL_CNT, self.volume_count.clone()),
            (FID_INPUT_PRICE_2, self.input_price2.clone()),
        ]
    }
}

impl Default for AfterHourBalanceRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AfterHourBalanceResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl AfterHourBalanceResponse {
    pub fn typed(&self) -> Result<Vec<AfterHourBalanceItem>> {
        parse_typed(
            self.output.clone(),
            "domestic stock after-hour balance ranking output",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn after_hour_balance(
        &self,
        access_token: &AccessToken,
        request: AfterHourBalanceRequest,
    ) -> Result<AfterHourBalanceResponse> {
        let (output, continuation) = get_output(
            self,
            access_token,
            Endpoint {
                path: AFTER_HOUR_BALANCE_PATH,
                tr_id: AFTER_HOUR_BALANCE_TR_ID,
            },
            request.params(),
            request.continuation.as_ref(),
            "domestic stock after-hour balance ranking",
        )
        .await?;

        Ok(AfterHourBalanceResponse {
            output,
            continuation,
        })
    }
}

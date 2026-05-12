use super::Service;
use super::common::{get_output, parse_typed};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::ranking::FluctuationItem;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use crate::rest::{PageCollection, PageLimit, PageStopReason};
use serde_json::Value;

pub const FLUCTUATION_PATH: &str = "/uapi/domestic-stock/v1/ranking/fluctuation";
pub const FLUCTUATION_TR_ID: &str = "FHPST01700000";

const FID_COND_MRKT_DIV_CODE: &str = "fid_cond_mrkt_div_code";
const FID_COND_SCR_DIV_CODE: &str = "fid_cond_scr_div_code";
const FID_INPUT_ISCD: &str = "fid_input_iscd";
const FID_RANK_SORT_CLS_CODE: &str = "fid_rank_sort_cls_code";
const FID_INPUT_CNT_1: &str = "fid_input_cnt_1";
const FID_PRC_CLS_CODE: &str = "fid_prc_cls_code";
const FID_INPUT_PRICE_1: &str = "fid_input_price_1";
const FID_INPUT_PRICE_2: &str = "fid_input_price_2";
const FID_VOL_CNT: &str = "fid_vol_cnt";
const FID_TRGT_CLS_CODE: &str = "fid_trgt_cls_code";
const FID_TRGT_EXLS_CLS_CODE: &str = "fid_trgt_exls_cls_code";
const FID_DIV_CLS_CODE: &str = "fid_div_cls_code";
const FID_RSFL_RATE1: &str = "fid_rsfl_rate1";
const FID_RSFL_RATE2: &str = "fid_rsfl_rate2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FluctuationRequest {
    pub market_division_code: String,
    pub screen_division_code: String,
    pub input_code: String,
    pub rank_sort_class_code: String,
    pub input_count: String,
    pub price_class_code: String,
    pub input_price1: String,
    pub input_price2: String,
    pub volume_count: String,
    pub target_class_code: String,
    pub target_exclusion_class_code: String,
    pub division_class_code: String,
    pub fluctuation_rate1: String,
    pub fluctuation_rate2: String,
    pub continuation: Option<Continuation>,
}

impl FluctuationRequest {
    pub fn new() -> Self {
        Self {
            market_division_code: "J".to_string(),
            screen_division_code: "20170".to_string(),
            input_code: "0000".to_string(),
            rank_sort_class_code: "0".to_string(),
            input_count: "0".to_string(),
            price_class_code: "0".to_string(),
            input_price1: String::new(),
            input_price2: String::new(),
            volume_count: String::new(),
            target_class_code: "0".to_string(),
            target_exclusion_class_code: "0".to_string(),
            division_class_code: "0".to_string(),
            fluctuation_rate1: String::new(),
            fluctuation_rate2: String::new(),
            continuation: None,
        }
    }

    pub fn with_market_division_code(mut self, value: impl Into<String>) -> Self {
        self.market_division_code = value.into();
        self
    }

    pub fn with_input_code(mut self, value: impl Into<String>) -> Self {
        self.input_code = value.into();
        self
    }

    pub fn with_rank_sort_class_code(mut self, value: impl Into<String>) -> Self {
        self.rank_sort_class_code = value.into();
        self
    }

    pub fn with_input_count(mut self, value: impl Into<String>) -> Self {
        self.input_count = value.into();
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

    pub fn with_target_class_code(mut self, value: impl Into<String>) -> Self {
        self.target_class_code = value.into();
        self
    }

    pub fn with_target_exclusion_class_code(mut self, value: impl Into<String>) -> Self {
        self.target_exclusion_class_code = value.into();
        self
    }

    pub fn with_division_class_code(mut self, value: impl Into<String>) -> Self {
        self.division_class_code = value.into();
        self
    }

    pub fn with_fluctuation_rate_range(
        mut self,
        lower_bound: impl Into<String>,
        upper_bound: impl Into<String>,
    ) -> Self {
        self.fluctuation_rate1 = lower_bound.into();
        self.fluctuation_rate2 = upper_bound.into();
        self
    }

    pub fn with_continuation(mut self, value: Continuation) -> Self {
        self.continuation = Some(value);
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        vec![
            (FID_RSFL_RATE2, self.fluctuation_rate2.clone()),
            (FID_COND_MRKT_DIV_CODE, self.market_division_code.clone()),
            (FID_COND_SCR_DIV_CODE, self.screen_division_code.clone()),
            (FID_INPUT_ISCD, self.input_code.clone()),
            (FID_RANK_SORT_CLS_CODE, self.rank_sort_class_code.clone()),
            (FID_INPUT_CNT_1, self.input_count.clone()),
            (FID_PRC_CLS_CODE, self.price_class_code.clone()),
            (FID_INPUT_PRICE_1, self.input_price1.clone()),
            (FID_INPUT_PRICE_2, self.input_price2.clone()),
            (FID_VOL_CNT, self.volume_count.clone()),
            (FID_TRGT_CLS_CODE, self.target_class_code.clone()),
            (
                FID_TRGT_EXLS_CLS_CODE,
                self.target_exclusion_class_code.clone(),
            ),
            (FID_DIV_CLS_CODE, self.division_class_code.clone()),
            (FID_RSFL_RATE1, self.fluctuation_rate1.clone()),
        ]
    }
}

impl Default for FluctuationRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FluctuationResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl FluctuationResponse {
    pub fn typed(&self) -> Result<Vec<FluctuationItem>> {
        parse_typed(
            self.output.clone(),
            "domestic stock fluctuation ranking output",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn fluctuation(
        &self,
        access_token: &AccessToken,
        request: FluctuationRequest,
    ) -> Result<FluctuationResponse> {
        let (output, continuation) = get_output(
            self,
            access_token,
            Endpoint {
                path: FLUCTUATION_PATH,
                tr_id: FLUCTUATION_TR_ID,
            },
            request.params(),
            request.continuation.as_ref(),
            "domestic stock fluctuation ranking",
        )
        .await?;

        Ok(FluctuationResponse {
            output,
            continuation,
        })
    }

    pub async fn fluctuation_pages(
        &self,
        access_token: &AccessToken,
        mut request: FluctuationRequest,
        limit: PageLimit,
    ) -> Result<PageCollection<FluctuationResponse, Continuation>> {
        let max_pages = limit.max_pages()?;
        let mut pages = Vec::new();

        loop {
            let response = match self.fluctuation(access_token, request.clone()).await {
                Ok(response) => response,
                Err(error) if error.is_rate_limited() => {
                    return Ok(PageCollection {
                        pages,
                        next: request.continuation,
                        stop_reason: PageStopReason::RateLimited { error },
                    });
                }
                Err(error) => return Err(error),
            };

            let next = response.continuation.next_request();
            pages.push(response);

            let Some(next) = next else {
                return Ok(PageCollection {
                    pages,
                    next: None,
                    stop_reason: PageStopReason::Exhausted,
                });
            };

            if max_pages.is_some_and(|max_pages| pages.len() >= max_pages) {
                return Ok(PageCollection {
                    pages,
                    next: Some(next),
                    stop_reason: PageStopReason::PageLimitReached,
                });
            }

            request = request.with_continuation(next);
        }
    }
}

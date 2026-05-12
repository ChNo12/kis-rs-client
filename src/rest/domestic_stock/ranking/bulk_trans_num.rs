use super::Service;
use super::common::{get_output, parse_typed};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::models::domestic_stock::ranking::BulkTransNumItem;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use crate::rest::{PageCollection, PageLimit, PageStopReason};
use serde_json::Value;

pub const BULK_TRANS_NUM_PATH: &str = "/uapi/domestic-stock/v1/ranking/bulk-trans-num";
pub const BULK_TRANS_NUM_TR_ID: &str = "FHKST190900C0";

const FID_APLY_RANG_PRC_2: &str = "fid_aply_rang_prc_2";
const FID_COND_MRKT_DIV_CODE: &str = "fid_cond_mrkt_div_code";
const FID_COND_SCR_DIV_CODE: &str = "fid_cond_scr_div_code";
const FID_INPUT_ISCD: &str = "fid_input_iscd";
const FID_RANK_SORT_CLS_CODE: &str = "fid_rank_sort_cls_code";
const FID_DIV_CLS_CODE: &str = "fid_div_cls_code";
const FID_INPUT_PRICE_1: &str = "fid_input_price_1";
const FID_APLY_RANG_PRC_1: &str = "fid_aply_rang_prc_1";
const FID_INPUT_ISCD_2: &str = "fid_input_iscd_2";
const FID_TRGT_EXLS_CLS_CODE: &str = "fid_trgt_exls_cls_code";
const FID_TRGT_CLS_CODE: &str = "fid_trgt_cls_code";
const FID_VOL_CNT: &str = "fid_vol_cnt";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkTransNumRequest {
    pub applied_range_price2: String,
    pub market_division_code: String,
    pub screen_division_code: String,
    pub input_code: String,
    pub rank_sort_class_code: String,
    pub division_class_code: String,
    pub input_price1: String,
    pub applied_range_price1: String,
    pub input_code2: String,
    pub target_exclusion_class_code: String,
    pub target_class_code: String,
    pub volume_count: String,
    pub continuation: Option<Continuation>,
}

impl BulkTransNumRequest {
    pub fn new() -> Self {
        Self {
            applied_range_price2: String::new(),
            market_division_code: "J".to_string(),
            screen_division_code: "11909".to_string(),
            input_code: "0000".to_string(),
            rank_sort_class_code: "0".to_string(),
            division_class_code: "0".to_string(),
            input_price1: String::new(),
            applied_range_price1: String::new(),
            input_code2: String::new(),
            target_exclusion_class_code: "0".to_string(),
            target_class_code: "0".to_string(),
            volume_count: String::new(),
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

    pub fn with_division_class_code(mut self, value: impl Into<String>) -> Self {
        self.division_class_code = value.into();
        self
    }

    pub fn with_input_price1(mut self, value: impl Into<String>) -> Self {
        self.input_price1 = value.into();
        self
    }

    pub fn with_applied_price_range(
        mut self,
        lower_bound: impl Into<String>,
        upper_bound: impl Into<String>,
    ) -> Self {
        self.applied_range_price1 = lower_bound.into();
        self.applied_range_price2 = upper_bound.into();
        self
    }

    pub fn with_input_code2(mut self, value: impl Into<String>) -> Self {
        self.input_code2 = value.into();
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

    pub fn with_continuation(mut self, value: Continuation) -> Self {
        self.continuation = Some(value);
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        vec![
            (FID_APLY_RANG_PRC_2, self.applied_range_price2.clone()),
            (FID_COND_MRKT_DIV_CODE, self.market_division_code.clone()),
            (FID_COND_SCR_DIV_CODE, self.screen_division_code.clone()),
            (FID_INPUT_ISCD, self.input_code.clone()),
            (FID_RANK_SORT_CLS_CODE, self.rank_sort_class_code.clone()),
            (FID_DIV_CLS_CODE, self.division_class_code.clone()),
            (FID_INPUT_PRICE_1, self.input_price1.clone()),
            (FID_APLY_RANG_PRC_1, self.applied_range_price1.clone()),
            (FID_INPUT_ISCD_2, self.input_code2.clone()),
            (
                FID_TRGT_EXLS_CLS_CODE,
                self.target_exclusion_class_code.clone(),
            ),
            (FID_TRGT_CLS_CODE, self.target_class_code.clone()),
            (FID_VOL_CNT, self.volume_count.clone()),
        ]
    }
}

impl Default for BulkTransNumRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BulkTransNumResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl BulkTransNumResponse {
    pub fn typed(&self) -> Result<Vec<BulkTransNumItem>> {
        parse_typed(
            self.output.clone(),
            "domestic stock bulk transaction number ranking output",
        )
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn bulk_trans_num(
        &self,
        access_token: &AccessToken,
        request: BulkTransNumRequest,
    ) -> Result<BulkTransNumResponse> {
        let (output, continuation) = get_output(
            self,
            access_token,
            Endpoint {
                path: BULK_TRANS_NUM_PATH,
                tr_id: BULK_TRANS_NUM_TR_ID,
            },
            request.params(),
            request.continuation.as_ref(),
            "domestic stock bulk transaction number ranking",
        )
        .await?;

        Ok(BulkTransNumResponse {
            output,
            continuation,
        })
    }

    pub async fn bulk_trans_num_pages(
        &self,
        access_token: &AccessToken,
        mut request: BulkTransNumRequest,
        limit: PageLimit,
    ) -> Result<PageCollection<BulkTransNumResponse, Continuation>> {
        let max_pages = limit.max_pages()?;
        let mut pages = Vec::new();

        loop {
            let response = match self.bulk_trans_num(access_token, request.clone()).await {
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

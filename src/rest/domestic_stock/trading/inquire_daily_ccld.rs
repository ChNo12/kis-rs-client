use serde_json::Value;

use super::Service;
use super::common::{
    CCLD_DVSN, CTX_AREA_FK100, CTX_AREA_NK100, EXCG_ID_DVSN_CD, INQR_DVSN, INQR_DVSN_1,
    INQR_DVSN_3, INQR_END_DT, INQR_STRT_DT, ODNO, ORD_GNO_BRNO, PDNO, SLL_BUY_DVSN_CD,
    get_output_pair, require_account_params, require_non_empty,
};
use crate::auth::AccessToken;
use crate::config::Environment;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use crate::rest::{PageCollection, PageLimit, PageStopReason};

pub const INQUIRE_DAILY_CCLD_PATH: &str = "/uapi/domestic-stock/v1/trading/inquire-daily-ccld";
pub const INQUIRE_DAILY_CCLD_REAL_INNER_TR_ID: &str = "TTTC0081R";
pub const INQUIRE_DAILY_CCLD_VIRTUAL_INNER_TR_ID: &str = "VTTC0081R";
pub const INQUIRE_DAILY_CCLD_REAL_BEFORE_TR_ID: &str = "CTSC9215R";
pub const INQUIRE_DAILY_CCLD_VIRTUAL_BEFORE_TR_ID: &str = "VTSC9215R";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InquireDailyCcldPeriod {
    Inner3Months,
    Before3Months,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireDailyCcldRequest {
    pub period: InquireDailyCcldPeriod,
    pub start_date: String,
    pub end_date: String,
    pub sell_buy_division_code: String,
    pub conclusion_division: String,
    pub inquiry_division: String,
    pub inquiry_division3: String,
    pub stock_code: String,
    pub order_branch_no: String,
    pub order_no: String,
    pub inquiry_division1: String,
    pub exchange_id_division_code: Option<String>,
    pub continuation: Option<Continuation>,
}

impl InquireDailyCcldRequest {
    pub fn new(
        period: InquireDailyCcldPeriod,
        start_date: impl Into<String>,
        end_date: impl Into<String>,
        sell_buy_division_code: impl Into<String>,
        conclusion_division: impl Into<String>,
        inquiry_division: impl Into<String>,
        inquiry_division3: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            period,
            start_date: require_non_empty(start_date, "start date")?,
            end_date: require_non_empty(end_date, "end date")?,
            sell_buy_division_code: require_non_empty(
                sell_buy_division_code,
                "sell buy division code",
            )?,
            conclusion_division: require_non_empty(conclusion_division, "conclusion division")?,
            inquiry_division: require_non_empty(inquiry_division, "inquiry division")?,
            inquiry_division3: require_non_empty(inquiry_division3, "inquiry division3")?,
            stock_code: String::new(),
            order_branch_no: String::new(),
            order_no: String::new(),
            inquiry_division1: String::new(),
            exchange_id_division_code: Some("KRX".to_string()),
            continuation: None,
        })
    }

    pub fn with_stock_code(mut self, value: impl Into<String>) -> Self {
        self.stock_code = value.into();
        self
    }

    pub fn with_order_branch_no(mut self, value: impl Into<String>) -> Self {
        self.order_branch_no = value.into();
        self
    }

    pub fn with_order_no(mut self, value: impl Into<String>) -> Self {
        self.order_no = value.into();
        self
    }

    pub fn with_inquiry_division1(mut self, value: impl Into<String>) -> Self {
        self.inquiry_division1 = value.into();
        self
    }

    pub fn with_exchange_id_division_code(mut self, value: impl Into<String>) -> Self {
        self.exchange_id_division_code = Some(value.into());
        self
    }

    pub fn without_exchange_id_division_code(mut self) -> Self {
        self.exchange_id_division_code = None;
        self
    }

    pub fn with_continuation(mut self, continuation: Continuation) -> Self {
        self.continuation = Some(continuation);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireDailyCcldResponse {
    pub output1: Value,
    pub output2: Value,
    pub continuation: Continuation,
}

pub const fn inquire_daily_ccld_tr_id(
    environment: Environment,
    period: InquireDailyCcldPeriod,
) -> &'static str {
    match (environment, period) {
        (Environment::Real, InquireDailyCcldPeriod::Inner3Months) => {
            INQUIRE_DAILY_CCLD_REAL_INNER_TR_ID
        }
        (Environment::Virtual, InquireDailyCcldPeriod::Inner3Months) => {
            INQUIRE_DAILY_CCLD_VIRTUAL_INNER_TR_ID
        }
        (Environment::Real, InquireDailyCcldPeriod::Before3Months) => {
            INQUIRE_DAILY_CCLD_REAL_BEFORE_TR_ID
        }
        (Environment::Virtual, InquireDailyCcldPeriod::Before3Months) => {
            INQUIRE_DAILY_CCLD_VIRTUAL_BEFORE_TR_ID
        }
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_daily_ccld(
        &self,
        access_token: &AccessToken,
        request: InquireDailyCcldRequest,
    ) -> Result<InquireDailyCcldResponse> {
        let mut params = require_account_params(self)?;
        params.extend([
            (INQR_STRT_DT, request.start_date),
            (INQR_END_DT, request.end_date),
            (SLL_BUY_DVSN_CD, request.sell_buy_division_code),
            (PDNO, request.stock_code),
            (CCLD_DVSN, request.conclusion_division),
            (INQR_DVSN, request.inquiry_division),
            (INQR_DVSN_3, request.inquiry_division3),
            (ORD_GNO_BRNO, request.order_branch_no),
            (ODNO, request.order_no),
            (INQR_DVSN_1, request.inquiry_division1),
            (
                CTX_AREA_FK100,
                request
                    .continuation
                    .as_ref()
                    .and_then(|continuation| continuation.ctx_area_fk.clone())
                    .unwrap_or_default(),
            ),
            (
                CTX_AREA_NK100,
                request
                    .continuation
                    .as_ref()
                    .and_then(|continuation| continuation.ctx_area_nk.clone())
                    .unwrap_or_default(),
            ),
        ]);

        if let Some(exchange_id_division_code) = request.exchange_id_division_code {
            params.push((EXCG_ID_DVSN_CD, exchange_id_division_code));
        }

        let response = get_output_pair(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_DAILY_CCLD_PATH,
                tr_id: inquire_daily_ccld_tr_id(self.client.config().environment, request.period),
            },
            params,
            request.continuation.as_ref(),
            "domestic stock daily order conclusion",
        )
        .await?;

        Ok(InquireDailyCcldResponse {
            output1: response.output1,
            output2: response.output2,
            continuation: response.continuation,
        })
    }

    pub async fn inquire_daily_ccld_pages(
        &self,
        access_token: &AccessToken,
        mut request: InquireDailyCcldRequest,
        limit: PageLimit,
    ) -> Result<PageCollection<InquireDailyCcldResponse, Continuation>> {
        let max_pages = limit.max_pages()?;
        let mut pages = Vec::new();

        loop {
            let response = match self.inquire_daily_ccld(access_token, request.clone()).await {
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

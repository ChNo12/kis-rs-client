use serde_json::Value;

use super::Service;
use super::common::{
    AFHR_FLPR_YN, CTX_AREA_FK100, CTX_AREA_NK100, FNCG_AMT_AUTO_RDPT_YN, FUND_STTL_ICLD_YN,
    INQR_DVSN, OFL_YN, PRCS_DVSN, UNPR_DVSN, env_tr_id, get_output_pair, require_account_params,
    require_non_empty,
};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use crate::rest::{PageCollection, PageLimit, PageStopReason};

pub const INQUIRE_BALANCE_PATH: &str = "/uapi/domestic-stock/v1/trading/inquire-balance";
pub const INQUIRE_BALANCE_REAL_TR_ID: &str = "TTTC8434R";
pub const INQUIRE_BALANCE_VIRTUAL_TR_ID: &str = "VTTC8434R";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireBalanceRequest {
    pub after_hour_single_price_yn: String,
    pub offline_yn: String,
    pub inquiry_division: String,
    pub unit_price_division: String,
    pub fund_settlement_included_yn: String,
    pub financing_amount_auto_redemption_yn: String,
    pub process_division: String,
    pub continuation: Option<Continuation>,
}

impl InquireBalanceRequest {
    pub fn new(
        after_hour_single_price_yn: impl Into<String>,
        inquiry_division: impl Into<String>,
        unit_price_division: impl Into<String>,
        fund_settlement_included_yn: impl Into<String>,
        financing_amount_auto_redemption_yn: impl Into<String>,
        process_division: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            after_hour_single_price_yn: require_non_empty(
                after_hour_single_price_yn,
                "after hour single price yn",
            )?,
            offline_yn: String::new(),
            inquiry_division: require_non_empty(inquiry_division, "inquiry division")?,
            unit_price_division: require_non_empty(unit_price_division, "unit price division")?,
            fund_settlement_included_yn: require_non_empty(
                fund_settlement_included_yn,
                "fund settlement included yn",
            )?,
            financing_amount_auto_redemption_yn: require_non_empty(
                financing_amount_auto_redemption_yn,
                "financing amount auto redemption yn",
            )?,
            process_division: require_non_empty(process_division, "process division")?,
            continuation: None,
        })
    }

    pub fn with_offline_yn(mut self, value: impl Into<String>) -> Self {
        self.offline_yn = value.into();
        self
    }

    pub fn with_continuation(mut self, continuation: Continuation) -> Self {
        self.continuation = Some(continuation);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireBalanceResponse {
    pub output1: Value,
    pub output2: Value,
    pub continuation: Continuation,
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_balance(
        &self,
        access_token: &AccessToken,
        request: InquireBalanceRequest,
    ) -> Result<InquireBalanceResponse> {
        let mut params = require_account_params(self)?;
        params.extend([
            (AFHR_FLPR_YN, request.after_hour_single_price_yn),
            (OFL_YN, request.offline_yn),
            (INQR_DVSN, request.inquiry_division),
            (UNPR_DVSN, request.unit_price_division),
            (FUND_STTL_ICLD_YN, request.fund_settlement_included_yn),
            (
                FNCG_AMT_AUTO_RDPT_YN,
                request.financing_amount_auto_redemption_yn,
            ),
            (PRCS_DVSN, request.process_division),
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

        let response = get_output_pair(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_BALANCE_PATH,
                tr_id: env_tr_id(
                    self.client.config().environment,
                    INQUIRE_BALANCE_REAL_TR_ID,
                    INQUIRE_BALANCE_VIRTUAL_TR_ID,
                ),
            },
            params,
            request.continuation.as_ref(),
            "domestic stock balance",
        )
        .await?;

        Ok(InquireBalanceResponse {
            output1: response.output1,
            output2: response.output2,
            continuation: response.continuation,
        })
    }

    pub async fn inquire_balance_pages(
        &self,
        access_token: &AccessToken,
        mut request: InquireBalanceRequest,
        limit: PageLimit,
    ) -> Result<PageCollection<InquireBalanceResponse, Continuation>> {
        let max_pages = limit.max_pages()?;
        let mut pages = Vec::new();

        loop {
            let response = match self.inquire_balance(access_token, request.clone()).await {
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

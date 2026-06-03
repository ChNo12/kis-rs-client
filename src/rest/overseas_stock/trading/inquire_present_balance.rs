use serde_json::Value;

use super::Service;
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::overseas_stock::Continuation;
use crate::rest::overseas_stock::common::{
    Endpoint, account_params, env_tr_id, get, require_non_empty, require_output_triple,
};
use crate::rest::{PageCollection, PageLimit, PageStopReason};

pub const INQUIRE_PRESENT_BALANCE_PATH: &str =
    "/uapi/overseas-stock/v1/trading/inquire-present-balance";
pub const INQUIRE_PRESENT_BALANCE_REAL_TR_ID: &str = "CTRP6504R";
pub const INQUIRE_PRESENT_BALANCE_VIRTUAL_TR_ID: &str = "VTRP6504R";

const WCRC_FRCR_DVSN_CD: &str = "WCRC_FRCR_DVSN_CD";
const NATN_CD: &str = "NATN_CD";
const TR_MKET_CD: &str = "TR_MKET_CD";
const INQR_DVSN_CD: &str = "INQR_DVSN_CD";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquirePresentBalanceRequest {
    pub won_foreign_division_code: String,
    pub country_code: String,
    pub market_code: String,
    pub inquiry_division_code: String,
    pub continuation: Option<Continuation>,
}

impl InquirePresentBalanceRequest {
    pub fn new(
        won_foreign_division_code: impl Into<String>,
        country_code: impl Into<String>,
        market_code: impl Into<String>,
        inquiry_division_code: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            won_foreign_division_code: require_non_empty(
                won_foreign_division_code,
                "won foreign division code",
            )?,
            country_code: require_non_empty(country_code, "country code")?,
            market_code: require_non_empty(market_code, "market code")?,
            inquiry_division_code: require_non_empty(
                inquiry_division_code,
                "inquiry division code",
            )?,
            continuation: None,
        })
    }

    pub fn with_continuation(mut self, continuation: Continuation) -> Self {
        self.continuation = Some(continuation);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquirePresentBalanceResponse {
    pub output1: Value,
    pub output2: Value,
    pub output3: Value,
    pub continuation: Continuation,
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_present_balance(
        &self,
        access_token: &AccessToken,
        request: InquirePresentBalanceRequest,
    ) -> Result<InquirePresentBalanceResponse> {
        let account = self.client.config().require_account()?;
        let mut params = account_params(account);
        params.extend([
            (WCRC_FRCR_DVSN_CD, request.won_foreign_division_code),
            (NATN_CD, request.country_code),
            (TR_MKET_CD, request.market_code),
            (INQR_DVSN_CD, request.inquiry_division_code),
        ]);

        let response = get(
            self.client,
            access_token,
            Endpoint {
                path: INQUIRE_PRESENT_BALANCE_PATH,
                tr_id: env_tr_id(
                    self.client.config().environment,
                    INQUIRE_PRESENT_BALANCE_REAL_TR_ID,
                    INQUIRE_PRESENT_BALANCE_VIRTUAL_TR_ID,
                ),
            },
            params,
            request.continuation.as_ref(),
            "overseas stock present balance",
        )
        .await?;
        let (output1, output2, output3, continuation) =
            require_output_triple(response, "overseas stock present balance")?;

        Ok(InquirePresentBalanceResponse {
            output1,
            output2,
            output3,
            continuation,
        })
    }

    pub async fn inquire_present_balance_pages(
        &self,
        access_token: &AccessToken,
        mut request: InquirePresentBalanceRequest,
        limit: PageLimit,
    ) -> Result<PageCollection<InquirePresentBalanceResponse, Continuation>> {
        let max_pages = limit.max_pages()?;
        let mut pages = Vec::new();

        loop {
            let response = match self
                .inquire_present_balance(access_token, request.clone())
                .await
            {
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

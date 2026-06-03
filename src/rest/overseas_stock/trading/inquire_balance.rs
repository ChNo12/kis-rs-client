use serde_json::Value;

use super::Service;
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::overseas_stock::Continuation;
use crate::rest::overseas_stock::common::{
    Endpoint, account_params, env_tr_id, get, require_non_empty, require_output_pair,
};
use crate::rest::{PageCollection, PageLimit, PageStopReason};

pub const INQUIRE_BALANCE_PATH: &str = "/uapi/overseas-stock/v1/trading/inquire-balance";
pub const INQUIRE_BALANCE_REAL_TR_ID: &str = "TTTS3012R";
pub const INQUIRE_BALANCE_VIRTUAL_TR_ID: &str = "VTTS3012R";

const OVRS_EXCG_CD: &str = "OVRS_EXCG_CD";
const TR_CRCY_CD: &str = "TR_CRCY_CD";
const CTX_AREA_FK200: &str = "CTX_AREA_FK200";
const CTX_AREA_NK200: &str = "CTX_AREA_NK200";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireBalanceRequest {
    pub exchange_code: String,
    pub transaction_currency_code: String,
    pub continuation: Option<Continuation>,
}

impl InquireBalanceRequest {
    pub fn new(
        exchange_code: impl Into<String>,
        transaction_currency_code: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            exchange_code: require_non_empty(exchange_code, "exchange code")?,
            transaction_currency_code: require_non_empty(
                transaction_currency_code,
                "transaction currency code",
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
        let account = self.client.config().require_account()?;
        let mut params = account_params(account);
        params.extend([
            (OVRS_EXCG_CD, request.exchange_code),
            (TR_CRCY_CD, request.transaction_currency_code),
            (
                CTX_AREA_FK200,
                request
                    .continuation
                    .as_ref()
                    .and_then(|continuation| continuation.ctx_area_fk.clone())
                    .unwrap_or_default(),
            ),
            (
                CTX_AREA_NK200,
                request
                    .continuation
                    .as_ref()
                    .and_then(|continuation| continuation.ctx_area_nk.clone())
                    .unwrap_or_default(),
            ),
        ]);

        let response = get(
            self.client,
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
            "overseas stock balance",
        )
        .await?;
        let (output1, output2, continuation) =
            require_output_pair(response, "overseas stock balance")?;

        Ok(InquireBalanceResponse {
            output1,
            output2,
            continuation,
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

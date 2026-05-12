use serde_json::Value;

use super::Service;
use super::common::{
    CTX_AREA_FK100, CTX_AREA_NK100, INQR_DVSN_1, INQR_DVSN_2, env_tr_id, get_output,
    require_account_params, require_non_empty,
};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;
use crate::rest::{PageCollection, PageLimit, PageStopReason};

pub const INQUIRE_PSBL_RVSECNCL_PATH: &str =
    "/uapi/domestic-stock/v1/trading/inquire-psbl-rvsecncl";
pub const INQUIRE_PSBL_RVSECNCL_TR_ID: &str = "TTTC0084R";
pub const INQUIRE_PSBL_RVSECNCL_VIRTUAL_TR_ID: &str = "VTTC0084R";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquirePsblRvsecnclRequest {
    pub inquiry_division1: String,
    pub inquiry_division2: String,
    pub continuation: Option<Continuation>,
}

impl InquirePsblRvsecnclRequest {
    pub fn new(
        inquiry_division1: impl Into<String>,
        inquiry_division2: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            inquiry_division1: require_non_empty(inquiry_division1, "inquiry division1")?,
            inquiry_division2: require_non_empty(inquiry_division2, "inquiry division2")?,
            continuation: None,
        })
    }

    pub fn with_continuation(mut self, continuation: Continuation) -> Self {
        self.continuation = Some(continuation);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquirePsblRvsecnclResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_psbl_rvsecncl(
        &self,
        access_token: &AccessToken,
        request: InquirePsblRvsecnclRequest,
    ) -> Result<InquirePsblRvsecnclResponse> {
        let mut params = require_account_params(self)?;
        params.extend([
            (INQR_DVSN_1, request.inquiry_division1),
            (INQR_DVSN_2, request.inquiry_division2),
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

        let response = get_output(
            self,
            access_token,
            Endpoint {
                path: INQUIRE_PSBL_RVSECNCL_PATH,
                tr_id: env_tr_id(
                    self.client.config().environment,
                    INQUIRE_PSBL_RVSECNCL_TR_ID,
                    INQUIRE_PSBL_RVSECNCL_VIRTUAL_TR_ID,
                ),
            },
            params,
            request.continuation.as_ref(),
            "domestic stock possible revise or cancel order",
        )
        .await?;

        Ok(InquirePsblRvsecnclResponse {
            output: response.output,
            continuation: response.continuation,
        })
    }

    pub async fn inquire_psbl_rvsecncl_pages(
        &self,
        access_token: &AccessToken,
        mut request: InquirePsblRvsecnclRequest,
        limit: PageLimit,
    ) -> Result<PageCollection<InquirePsblRvsecnclResponse, Continuation>> {
        let max_pages = limit.max_pages()?;
        let mut pages = Vec::new();

        loop {
            let response = match self
                .inquire_psbl_rvsecncl(access_token, request.clone())
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

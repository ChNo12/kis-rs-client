use serde_json::Value;

use super::{Service, SingleOutputResponse};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::overseas_stock::Continuation;
use crate::rest::overseas_stock::common::{
    Endpoint, OverseasExchange, account_params, env_tr_id, get, require_non_empty, require_output,
};

pub const INQUIRE_CCNL_PATH: &str = "/uapi/overseas-stock/v1/trading/inquire-ccnl";
pub const INQUIRE_CCNL_REAL_TR_ID: &str = "TTTS3035R";
pub const INQUIRE_CCNL_MOCK_TR_ID: &str = "VTTS3035R";

const PDNO: &str = "PDNO";
const ORD_STRT_DT: &str = "ORD_STRT_DT";
const ORD_END_DT: &str = "ORD_END_DT";
const SLL_BUY_DVSN: &str = "SLL_BUY_DVSN";
const CCLD_NCCS_DVSN: &str = "CCLD_NCCS_DVSN";
const OVRS_EXCG_CD: &str = "OVRS_EXCG_CD";
const SORT_SQN: &str = "SORT_SQN";
const ORD_DT: &str = "ORD_DT";
const ORD_GNO_BRNO: &str = "ORD_GNO_BRNO";
const ODNO: &str = "ODNO";
const CTX_AREA_NK200: &str = "CTX_AREA_NK200";
const CTX_AREA_FK200: &str = "CTX_AREA_FK200";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InquireCcnlRequest {
    pub stock_code: String,
    pub order_start_date: String,
    pub order_end_date: String,
    pub sell_buy_division: String,
    pub conclusion_or_unfilled_division: String,
    pub exchange: Option<OverseasExchange>,
    pub sort_sequence: String,
    pub order_date: String,
    pub order_branch_no: String,
    pub order_no: String,
    pub continuation: Option<Continuation>,
}

impl InquireCcnlRequest {
    pub fn new(
        order_start_date: impl Into<String>,
        order_end_date: impl Into<String>,
        sell_buy_division: impl Into<String>,
        conclusion_or_unfilled_division: impl Into<String>,
        sort_sequence: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            stock_code: String::new(),
            order_start_date: require_non_empty(order_start_date, "order start date")?,
            order_end_date: require_non_empty(order_end_date, "order end date")?,
            sell_buy_division: require_non_empty(sell_buy_division, "sell buy division")?,
            conclusion_or_unfilled_division: require_non_empty(
                conclusion_or_unfilled_division,
                "conclusion or unfilled division",
            )?,
            exchange: Some(OverseasExchange::Nasdaq),
            sort_sequence: require_non_empty(sort_sequence, "sort sequence")?,
            order_date: String::new(),
            order_branch_no: String::new(),
            order_no: String::new(),
            continuation: None,
        })
    }

    pub fn with_stock_code(mut self, value: impl Into<String>) -> Self {
        self.stock_code = value.into();
        self
    }

    pub fn with_exchange(mut self, exchange: OverseasExchange) -> Self {
        self.exchange = Some(exchange);
        self
    }

    pub fn without_exchange(mut self) -> Self {
        self.exchange = None;
        self
    }

    pub fn with_order_date(mut self, value: impl Into<String>) -> Self {
        self.order_date = value.into();
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

    pub fn with_continuation(mut self, continuation: Continuation) -> Self {
        self.continuation = Some(continuation);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InquireCcnlResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn inquire_ccnl(
        &self,
        access_token: &AccessToken,
        request: InquireCcnlRequest,
    ) -> Result<InquireCcnlResponse> {
        let account = self.client.config().require_account()?;
        let mut params = account_params(account);
        params.extend([
            (PDNO, request.stock_code),
            (ORD_STRT_DT, request.order_start_date),
            (ORD_END_DT, request.order_end_date),
            (SLL_BUY_DVSN, request.sell_buy_division),
            (CCLD_NCCS_DVSN, request.conclusion_or_unfilled_division),
            (
                OVRS_EXCG_CD,
                request
                    .exchange
                    .map(|exchange| exchange.as_str().to_string())
                    .unwrap_or_default(),
            ),
            (SORT_SQN, request.sort_sequence),
            (ORD_DT, request.order_date),
            (ORD_GNO_BRNO, request.order_branch_no),
            (ODNO, request.order_no),
            (
                CTX_AREA_NK200,
                request
                    .continuation
                    .as_ref()
                    .and_then(|continuation| continuation.ctx_area_nk.clone())
                    .unwrap_or_default(),
            ),
            (
                CTX_AREA_FK200,
                request
                    .continuation
                    .as_ref()
                    .and_then(|continuation| continuation.ctx_area_fk.clone())
                    .unwrap_or_default(),
            ),
        ]);

        let response = get(
            self.client,
            access_token,
            Endpoint {
                path: INQUIRE_CCNL_PATH,
                tr_id: env_tr_id(
                    self.client.config().environment,
                    INQUIRE_CCNL_REAL_TR_ID,
                    INQUIRE_CCNL_MOCK_TR_ID,
                ),
            },
            params,
            request.continuation.as_ref(),
            "overseas stock order conclusion",
        )
        .await?;
        let response = into_single_output_response(response, "overseas stock order conclusion")?;

        Ok(InquireCcnlResponse {
            output: response.output,
            continuation: response.continuation,
        })
    }
}

fn into_single_output_response(
    response: crate::rest::overseas_stock::common::RawResponse,
    parse_context: &'static str,
) -> Result<SingleOutputResponse> {
    let (output, continuation) = require_output(response, parse_context)?;

    Ok(SingleOutputResponse {
        output,
        continuation,
    })
}

use serde_json::Value;

use super::{Service, SingleOutputResponse};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::overseas_stock::Continuation;
use crate::rest::overseas_stock::common::{
    Endpoint, OverseasExchange, OverseasStockCode, account_params, env_tr_id, post,
    require_non_empty, require_output,
};

pub const ORDER_RVSECNCL_PATH: &str = "/uapi/overseas-stock/v1/trading/order-rvsecncl";
pub const ORDER_RVSECNCL_REAL_TR_ID: &str = "TTTT1004U";
pub const ORDER_RVSECNCL_VIRTUAL_TR_ID: &str = "VTTT1004U";

const OVRS_EXCG_CD: &str = "OVRS_EXCG_CD";
const PDNO: &str = "PDNO";
const ORGN_ODNO: &str = "ORGN_ODNO";
const RVSE_CNCL_DVSN_CD: &str = "RVSE_CNCL_DVSN_CD";
const ORD_QTY: &str = "ORD_QTY";
const OVRS_ORD_UNPR: &str = "OVRS_ORD_UNPR";
const MGCO_APTM_ODNO: &str = "MGCO_APTM_ODNO";
const ORD_SVR_DVSN_CD: &str = "ORD_SVR_DVSN_CD";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviseCancel {
    Revise,
    Cancel,
}

impl ReviseCancel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Revise => "01",
            Self::Cancel => "02",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderRvsecnclRequest {
    pub exchange: OverseasExchange,
    pub stock_code: OverseasStockCode,
    pub original_order_no: String,
    pub revise_cancel: ReviseCancel,
    pub order_quantity: String,
    pub overseas_order_unit_price: String,
    pub management_company_order_no: String,
    pub order_server_division_code: String,
}

impl OrderRvsecnclRequest {
    pub fn new(
        exchange: OverseasExchange,
        stock_code: OverseasStockCode,
        original_order_no: impl Into<String>,
        revise_cancel: ReviseCancel,
        order_quantity: impl Into<String>,
        overseas_order_unit_price: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            exchange,
            stock_code,
            original_order_no: require_non_empty(original_order_no, "original order no")?,
            revise_cancel,
            order_quantity: require_non_empty(order_quantity, "order quantity")?,
            overseas_order_unit_price: require_non_empty(
                overseas_order_unit_price,
                "overseas order unit price",
            )?,
            management_company_order_no: String::new(),
            order_server_division_code: "0".to_string(),
        })
    }

    pub fn revise(
        exchange: OverseasExchange,
        stock_code: OverseasStockCode,
        original_order_no: impl Into<String>,
        order_quantity: impl Into<String>,
        overseas_order_unit_price: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            exchange,
            stock_code,
            original_order_no,
            ReviseCancel::Revise,
            order_quantity,
            overseas_order_unit_price,
        )
    }

    pub fn cancel(
        exchange: OverseasExchange,
        stock_code: OverseasStockCode,
        original_order_no: impl Into<String>,
        order_quantity: impl Into<String>,
        overseas_order_unit_price: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            exchange,
            stock_code,
            original_order_no,
            ReviseCancel::Cancel,
            order_quantity,
            overseas_order_unit_price,
        )
    }

    pub fn with_management_company_order_no(mut self, value: impl Into<String>) -> Self {
        self.management_company_order_no = value.into();
        self
    }

    pub fn with_order_server_division_code(mut self, value: impl Into<String>) -> Self {
        self.order_server_division_code = value.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderRvsecnclResponse {
    pub output: Value,
    pub continuation: Continuation,
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn order_rvsecncl(
        &self,
        access_token: &AccessToken,
        request: OrderRvsecnclRequest,
    ) -> Result<OrderRvsecnclResponse> {
        self.client.config().require_ordering_allowed()?;
        let account = self.client.config().require_account()?;
        let mut body = account_params(account);
        body.extend([
            (OVRS_EXCG_CD, request.exchange.as_str().to_string()),
            (PDNO, request.stock_code.as_str().to_string()),
            (ORGN_ODNO, request.original_order_no),
            (
                RVSE_CNCL_DVSN_CD,
                request.revise_cancel.as_str().to_string(),
            ),
            (ORD_QTY, request.order_quantity),
            (OVRS_ORD_UNPR, request.overseas_order_unit_price),
            (MGCO_APTM_ODNO, request.management_company_order_no),
            (ORD_SVR_DVSN_CD, request.order_server_division_code),
        ]);

        let response = post(
            self.client,
            access_token,
            Endpoint {
                path: ORDER_RVSECNCL_PATH,
                tr_id: env_tr_id(
                    self.client.config().environment,
                    ORDER_RVSECNCL_REAL_TR_ID,
                    ORDER_RVSECNCL_VIRTUAL_TR_ID,
                ),
            },
            body,
            "overseas stock revise or cancel order",
        )
        .await?;
        let response =
            into_single_output_response(response, "overseas stock revise or cancel order")?;

        Ok(OrderRvsecnclResponse {
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

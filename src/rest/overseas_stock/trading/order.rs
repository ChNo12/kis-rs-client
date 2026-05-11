use serde_json::Value;

use super::{Service, SingleOutputResponse};
use crate::auth::AccessToken;
use crate::config::Environment;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::overseas_stock::Continuation;
use crate::rest::overseas_stock::common::{
    Endpoint, OrderSide, OverseasExchange, OverseasStockCode, account_params, post,
    require_non_empty, require_output,
};

pub const ORDER_PATH: &str = "/uapi/overseas-stock/v1/trading/order";
pub const ORDER_REAL_BUY_TR_ID: &str = "TTTT1002U";
pub const ORDER_MOCK_BUY_TR_ID: &str = "VTTT1002U";
pub const ORDER_REAL_SELL_TR_ID: &str = "TTTT1006U";
pub const ORDER_MOCK_SELL_TR_ID: &str = "VTTT1006U";

const OVRS_EXCG_CD: &str = "OVRS_EXCG_CD";
const PDNO: &str = "PDNO";
const ORD_QTY: &str = "ORD_QTY";
const OVRS_ORD_UNPR: &str = "OVRS_ORD_UNPR";
const CTAC_TLNO: &str = "CTAC_TLNO";
const MGCO_APTM_ODNO: &str = "MGCO_APTM_ODNO";
const SLL_TYPE: &str = "SLL_TYPE";
const ORD_SVR_DVSN_CD: &str = "ORD_SVR_DVSN_CD";
const ORD_DVSN: &str = "ORD_DVSN";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderRequest {
    pub side: OrderSide,
    pub exchange: OverseasExchange,
    pub stock_code: OverseasStockCode,
    pub order_quantity: String,
    pub overseas_order_unit_price: String,
    pub contact_phone_no: String,
    pub management_company_order_no: String,
    pub order_server_division_code: String,
    pub order_division: String,
}

impl OrderRequest {
    pub fn new(
        side: OrderSide,
        exchange: OverseasExchange,
        stock_code: OverseasStockCode,
        order_quantity: impl Into<String>,
        overseas_order_unit_price: impl Into<String>,
        order_division: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            side,
            exchange,
            stock_code,
            order_quantity: require_non_empty(order_quantity, "order quantity")?,
            overseas_order_unit_price: require_non_empty(
                overseas_order_unit_price,
                "overseas order unit price",
            )?,
            contact_phone_no: String::new(),
            management_company_order_no: String::new(),
            order_server_division_code: "0".to_string(),
            order_division: require_non_empty(order_division, "order division")?,
        })
    }

    pub fn buy(
        exchange: OverseasExchange,
        stock_code: OverseasStockCode,
        order_quantity: impl Into<String>,
        overseas_order_unit_price: impl Into<String>,
        order_division: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            OrderSide::Buy,
            exchange,
            stock_code,
            order_quantity,
            overseas_order_unit_price,
            order_division,
        )
    }

    pub fn sell(
        exchange: OverseasExchange,
        stock_code: OverseasStockCode,
        order_quantity: impl Into<String>,
        overseas_order_unit_price: impl Into<String>,
        order_division: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            OrderSide::Sell,
            exchange,
            stock_code,
            order_quantity,
            overseas_order_unit_price,
            order_division,
        )
    }

    pub fn with_contact_phone_no(mut self, value: impl Into<String>) -> Self {
        self.contact_phone_no = value.into();
        self
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
pub struct OrderResponse {
    pub output: Value,
    pub continuation: Continuation,
}

pub const fn order_tr_id(environment: Environment, side: OrderSide) -> &'static str {
    match (environment, side) {
        (Environment::Real, OrderSide::Buy) => ORDER_REAL_BUY_TR_ID,
        (Environment::Mock, OrderSide::Buy) => ORDER_MOCK_BUY_TR_ID,
        (Environment::Real, OrderSide::Sell) => ORDER_REAL_SELL_TR_ID,
        (Environment::Mock, OrderSide::Sell) => ORDER_MOCK_SELL_TR_ID,
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn order(
        &self,
        access_token: &AccessToken,
        request: OrderRequest,
    ) -> Result<OrderResponse> {
        self.client.config().require_ordering_allowed()?;
        let account = self.client.config().require_account()?;
        let mut body = account_params(account);
        body.extend([
            (OVRS_EXCG_CD, request.exchange.as_str().to_string()),
            (PDNO, request.stock_code.as_str().to_string()),
            (ORD_QTY, request.order_quantity),
            (OVRS_ORD_UNPR, request.overseas_order_unit_price),
            (CTAC_TLNO, request.contact_phone_no),
            (MGCO_APTM_ODNO, request.management_company_order_no),
            (
                SLL_TYPE,
                if request.side == OrderSide::Sell {
                    "00"
                } else {
                    ""
                }
                .to_string(),
            ),
            (ORD_SVR_DVSN_CD, request.order_server_division_code),
            (ORD_DVSN, request.order_division),
        ]);

        let response = post(
            self.client,
            access_token,
            Endpoint {
                path: ORDER_PATH,
                tr_id: order_tr_id(self.client.config().environment, request.side),
            },
            body,
            "overseas stock order",
        )
        .await?;
        let response = into_single_output_response(response, "overseas stock order")?;

        Ok(OrderResponse {
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

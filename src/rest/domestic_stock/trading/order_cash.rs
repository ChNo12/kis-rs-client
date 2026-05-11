use serde_json::Value;

use super::Service;
use super::common::{
    CNDT_PRIC, EXCG_ID_DVSN_CD, ORD_DVSN, ORD_QTY, ORD_UNPR, OrderSide, PDNO, SLL_TYPE,
    account_params, post_output, require_non_empty,
};
use crate::auth::AccessToken;
use crate::config::Environment;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::{Endpoint, StockCode};

pub const ORDER_CASH_PATH: &str = "/uapi/domestic-stock/v1/trading/order-cash";
pub const ORDER_CASH_REAL_SELL_TR_ID: &str = "TTTC0011U";
pub const ORDER_CASH_REAL_BUY_TR_ID: &str = "TTTC0012U";
pub const ORDER_CASH_MOCK_SELL_TR_ID: &str = "VTTC0011U";
pub const ORDER_CASH_MOCK_BUY_TR_ID: &str = "VTTC0012U";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderCashRequest {
    pub side: OrderSide,
    pub stock_code: StockCode,
    pub order_division: String,
    pub order_quantity: String,
    pub order_unit_price: String,
    pub exchange_id_division_code: String,
    pub sell_type: String,
    pub condition_price: String,
}

impl OrderCashRequest {
    pub fn new(
        side: OrderSide,
        stock_code: StockCode,
        order_division: impl Into<String>,
        order_quantity: impl Into<String>,
        order_unit_price: impl Into<String>,
        exchange_id_division_code: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            side,
            stock_code,
            order_division: require_non_empty(order_division, "order division")?,
            order_quantity: require_non_empty(order_quantity, "order quantity")?,
            order_unit_price: require_non_empty(order_unit_price, "order unit price")?,
            exchange_id_division_code: require_non_empty(
                exchange_id_division_code,
                "exchange id division code",
            )?,
            sell_type: String::new(),
            condition_price: String::new(),
        })
    }

    pub fn buy(
        stock_code: StockCode,
        order_division: impl Into<String>,
        order_quantity: impl Into<String>,
        order_unit_price: impl Into<String>,
        exchange_id_division_code: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            OrderSide::Buy,
            stock_code,
            order_division,
            order_quantity,
            order_unit_price,
            exchange_id_division_code,
        )
    }

    pub fn sell(
        stock_code: StockCode,
        order_division: impl Into<String>,
        order_quantity: impl Into<String>,
        order_unit_price: impl Into<String>,
        exchange_id_division_code: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            OrderSide::Sell,
            stock_code,
            order_division,
            order_quantity,
            order_unit_price,
            exchange_id_division_code,
        )
    }

    pub fn with_sell_type(mut self, value: impl Into<String>) -> Self {
        self.sell_type = value.into();
        self
    }

    pub fn with_condition_price(mut self, value: impl Into<String>) -> Self {
        self.condition_price = value.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderCashResponse {
    pub output: Value,
    pub continuation: Continuation,
}

pub const fn order_cash_tr_id(environment: Environment, side: OrderSide) -> &'static str {
    match (environment, side) {
        (Environment::Real, OrderSide::Sell) => ORDER_CASH_REAL_SELL_TR_ID,
        (Environment::Real, OrderSide::Buy) => ORDER_CASH_REAL_BUY_TR_ID,
        (Environment::Mock, OrderSide::Sell) => ORDER_CASH_MOCK_SELL_TR_ID,
        (Environment::Mock, OrderSide::Buy) => ORDER_CASH_MOCK_BUY_TR_ID,
    }
}

impl<T: HttpClient> Service<'_, T> {
    pub async fn order_cash(
        &self,
        access_token: &AccessToken,
        request: OrderCashRequest,
    ) -> Result<OrderCashResponse> {
        let account = self.client.config().require_account()?;
        let mut body = account_params(account);
        body.extend([
            (PDNO, request.stock_code.as_str().to_string()),
            (ORD_DVSN, request.order_division),
            (ORD_QTY, request.order_quantity),
            (ORD_UNPR, request.order_unit_price),
            (EXCG_ID_DVSN_CD, request.exchange_id_division_code),
            (SLL_TYPE, request.sell_type),
            (CNDT_PRIC, request.condition_price),
        ]);

        let response = post_output(
            self,
            access_token,
            Endpoint {
                path: ORDER_CASH_PATH,
                tr_id: order_cash_tr_id(self.client.config().environment, request.side),
            },
            body,
            "domestic stock cash order",
        )
        .await?;

        Ok(OrderCashResponse {
            output: response.output,
            continuation: response.continuation,
        })
    }
}

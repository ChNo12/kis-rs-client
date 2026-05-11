use serde_json::Value;

use super::Service;
use super::common::{
    AllQuantityOrder, CNDT_PRIC, EXCG_ID_DVSN_CD, KRX_FWDG_ORD_ORGNO, ORD_DVSN, ORD_QTY, ORD_UNPR,
    ORGN_ODNO, QTY_ALL_ORD_YN, RVSE_CNCL_DVSN_CD, ReviseCancel, account_params, env_tr_id,
    post_output, require_non_empty,
};
use crate::auth::AccessToken;
use crate::error::Result;
use crate::http::HttpClient;
use crate::rest::domestic_stock::Continuation;
use crate::rest::domestic_stock::common::Endpoint;

pub const ORDER_RVSECNCL_PATH: &str = "/uapi/domestic-stock/v1/trading/order-rvsecncl";
pub const ORDER_RVSECNCL_REAL_TR_ID: &str = "TTTC0013U";
pub const ORDER_RVSECNCL_MOCK_TR_ID: &str = "VTTC0013U";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderRvsecnclRequest {
    pub krx_forwarding_order_org_no: String,
    pub original_order_no: String,
    pub order_division: String,
    pub revise_cancel: ReviseCancel,
    pub order_quantity: String,
    pub order_unit_price: String,
    pub all_quantity_order: AllQuantityOrder,
    pub exchange_id_division_code: String,
    pub condition_price: Option<String>,
}

impl OrderRvsecnclRequest {
    pub fn revise(
        krx_forwarding_order_org_no: impl Into<String>,
        original_order_no: impl Into<String>,
        order_division: impl Into<String>,
        order_quantity: impl Into<String>,
        order_unit_price: impl Into<String>,
        all_quantity_order: AllQuantityOrder,
        exchange_id_division_code: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            krx_forwarding_order_org_no: require_non_empty(
                krx_forwarding_order_org_no,
                "krx forwarding order org no",
            )?,
            original_order_no: require_non_empty(original_order_no, "original order no")?,
            order_division: require_non_empty(order_division, "order division")?,
            revise_cancel: ReviseCancel::Revise,
            order_quantity: require_non_empty(order_quantity, "order quantity")?,
            order_unit_price: require_non_empty(order_unit_price, "order unit price")?,
            all_quantity_order,
            exchange_id_division_code: require_non_empty(
                exchange_id_division_code,
                "exchange id division code",
            )?,
            condition_price: None,
        })
    }

    pub fn cancel(
        krx_forwarding_order_org_no: impl Into<String>,
        original_order_no: impl Into<String>,
        order_division: impl Into<String>,
        order_quantity: impl Into<String>,
        order_unit_price: impl Into<String>,
        all_quantity_order: AllQuantityOrder,
        exchange_id_division_code: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            krx_forwarding_order_org_no: require_non_empty(
                krx_forwarding_order_org_no,
                "krx forwarding order org no",
            )?,
            original_order_no: require_non_empty(original_order_no, "original order no")?,
            order_division: require_non_empty(order_division, "order division")?,
            revise_cancel: ReviseCancel::Cancel,
            order_quantity: require_non_empty(order_quantity, "order quantity")?,
            order_unit_price: require_non_empty(order_unit_price, "order unit price")?,
            all_quantity_order,
            exchange_id_division_code: require_non_empty(
                exchange_id_division_code,
                "exchange id division code",
            )?,
            condition_price: None,
        })
    }

    pub fn with_condition_price(mut self, value: impl Into<String>) -> Self {
        self.condition_price = Some(value.into());
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
        let account = self.client.config().require_account()?;
        let mut body = account_params(account);
        body.extend([
            (KRX_FWDG_ORD_ORGNO, request.krx_forwarding_order_org_no),
            (ORGN_ODNO, request.original_order_no),
            (ORD_DVSN, request.order_division),
            (
                RVSE_CNCL_DVSN_CD,
                request.revise_cancel.as_str().to_string(),
            ),
            (ORD_QTY, request.order_quantity),
            (ORD_UNPR, request.order_unit_price),
            (
                QTY_ALL_ORD_YN,
                request.all_quantity_order.as_str().to_string(),
            ),
            (EXCG_ID_DVSN_CD, request.exchange_id_division_code),
        ]);

        if let Some(condition_price) = request.condition_price {
            body.push((CNDT_PRIC, condition_price));
        }

        let response = post_output(
            self,
            access_token,
            Endpoint {
                path: ORDER_RVSECNCL_PATH,
                tr_id: env_tr_id(
                    self.client.config().environment,
                    ORDER_RVSECNCL_REAL_TR_ID,
                    ORDER_RVSECNCL_MOCK_TR_ID,
                ),
            },
            body,
            "domestic stock revise or cancel order",
        )
        .await?;

        Ok(OrderRvsecnclResponse {
            output: response.output,
            continuation: response.continuation,
        })
    }
}

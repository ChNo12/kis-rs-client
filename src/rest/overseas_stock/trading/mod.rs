mod inquire_ccnl;
mod order;
mod order_rvsecncl;

pub use inquire_ccnl::{
    INQUIRE_CCNL_MOCK_TR_ID, INQUIRE_CCNL_PATH, INQUIRE_CCNL_REAL_TR_ID, InquireCcnlRequest,
    InquireCcnlResponse,
};
pub use order::{
    ORDER_MOCK_BUY_TR_ID, ORDER_MOCK_SELL_TR_ID, ORDER_PATH, ORDER_REAL_BUY_TR_ID,
    ORDER_REAL_SELL_TR_ID, OrderRequest, OrderResponse, order_tr_id,
};
pub use order_rvsecncl::{
    ORDER_RVSECNCL_MOCK_TR_ID, ORDER_RVSECNCL_PATH, ORDER_RVSECNCL_REAL_TR_ID,
    OrderRvsecnclRequest, OrderRvsecnclResponse, ReviseCancel,
};
use serde_json::Value;

use crate::client::Client;
use crate::rest::overseas_stock::Continuation;

#[derive(Clone, Copy, Debug)]
pub struct Service<'a, T> {
    pub(crate) client: &'a Client<T>,
}

impl<'a, T> Service<'a, T> {
    pub(crate) fn new(client: &'a Client<T>) -> Self {
        Self { client }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SingleOutputResponse {
    pub output: Value,
    pub continuation: Continuation,
}

#[cfg(test)]
mod tests;

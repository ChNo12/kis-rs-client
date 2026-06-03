mod common;
mod inquire_balance;
mod inquire_daily_ccld;
mod inquire_psbl_rvsecncl;
mod order_cash;
mod order_rvsecncl;

pub use common::{AllQuantityOrder, OrderSide, ReviseCancel};
pub use inquire_balance::{
    INQUIRE_BALANCE_PATH, INQUIRE_BALANCE_REAL_TR_ID, INQUIRE_BALANCE_VIRTUAL_TR_ID,
    InquireBalanceRequest, InquireBalanceResponse,
};
pub use inquire_daily_ccld::{
    INQUIRE_DAILY_CCLD_PATH, INQUIRE_DAILY_CCLD_REAL_BEFORE_TR_ID,
    INQUIRE_DAILY_CCLD_REAL_INNER_TR_ID, INQUIRE_DAILY_CCLD_VIRTUAL_BEFORE_TR_ID,
    INQUIRE_DAILY_CCLD_VIRTUAL_INNER_TR_ID, InquireDailyCcldPeriod, InquireDailyCcldRequest,
    InquireDailyCcldResponse, inquire_daily_ccld_tr_id,
};
pub use inquire_psbl_rvsecncl::{
    INQUIRE_PSBL_RVSECNCL_PATH, INQUIRE_PSBL_RVSECNCL_TR_ID, INQUIRE_PSBL_RVSECNCL_VIRTUAL_TR_ID,
    InquirePsblRvsecnclRequest, InquirePsblRvsecnclResponse,
};
pub use order_cash::{
    ORDER_CASH_PATH, ORDER_CASH_REAL_BUY_TR_ID, ORDER_CASH_REAL_SELL_TR_ID,
    ORDER_CASH_VIRTUAL_BUY_TR_ID, ORDER_CASH_VIRTUAL_SELL_TR_ID, OrderCashRequest,
    OrderCashResponse, order_cash_tr_id,
};
pub use order_rvsecncl::{
    ORDER_RVSECNCL_PATH, ORDER_RVSECNCL_REAL_TR_ID, ORDER_RVSECNCL_VIRTUAL_TR_ID,
    OrderRvsecnclRequest, OrderRvsecnclResponse,
};
use serde_json::Value;

use crate::client::Client;
use crate::rest::domestic_stock::Continuation;

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

#[derive(Clone, Debug, PartialEq)]
pub struct DoubleOutputResponse {
    pub output1: Value,
    pub output2: Value,
    pub continuation: Continuation,
}

#[cfg(test)]
mod tests;

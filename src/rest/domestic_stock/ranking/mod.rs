mod after_hour_balance;
mod bulk_trans_num;
mod common;
mod fluctuation;

pub use after_hour_balance::{
    AFTER_HOUR_BALANCE_PATH, AFTER_HOUR_BALANCE_TR_ID, AfterHourBalanceRequest,
    AfterHourBalanceResponse,
};
pub use bulk_trans_num::{
    BULK_TRANS_NUM_PATH, BULK_TRANS_NUM_TR_ID, BulkTransNumRequest, BulkTransNumResponse,
};
pub use fluctuation::{
    FLUCTUATION_PATH, FLUCTUATION_TR_ID, FluctuationRequest, FluctuationResponse,
};

use crate::client::Client;

#[derive(Clone, Copy, Debug)]
pub struct Service<'a, T> {
    pub(crate) client: &'a Client<T>,
}

impl<'a, T> Service<'a, T> {
    pub(crate) fn new(client: &'a Client<T>) -> Self {
        Self { client }
    }
}

#[cfg(test)]
mod tests;

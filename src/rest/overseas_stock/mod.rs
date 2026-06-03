mod common;
pub mod trading;

pub use common::{Continuation, OrderSide, OverseasExchange, OverseasStockCode};
pub use trading::{
    InquireBalanceRequest, InquireBalanceResponse, InquireCcnlRequest, InquireCcnlResponse,
    InquirePresentBalanceRequest, InquirePresentBalanceResponse, OrderRequest, OrderResponse,
    OrderRvsecnclRequest, OrderRvsecnclResponse, ReviseCancel,
};

use crate::client::Client;

#[derive(Clone, Copy, Debug)]
pub struct Service<'a, T> {
    client: &'a Client<T>,
}

impl<'a, T> Service<'a, T> {
    pub(crate) fn new(client: &'a Client<T>) -> Self {
        Self { client }
    }

    pub fn trading(self) -> trading::Service<'a, T> {
        trading::Service::new(self.client)
    }
}

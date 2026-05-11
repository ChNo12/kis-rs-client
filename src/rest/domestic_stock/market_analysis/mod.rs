mod capture_up_low_price;
mod common;

pub use capture_up_low_price::{
    CAPTURE_UP_LOW_PRICE_PATH, CAPTURE_UP_LOW_PRICE_TR_ID, CaptureUpLowPriceRequest,
    CaptureUpLowPriceResponse,
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

mod common;
mod inquire_asking_price_exp_ccn;
mod inquire_ccnl;
mod inquire_daily_item_chart_price;
mod inquire_daily_price;
mod inquire_price;
mod inquire_time_daily_chart_price;
mod inquire_time_item_chart_price;

pub use inquire_asking_price_exp_ccn::{
    INQUIRE_ASKING_PRICE_EXP_CCN_PATH, INQUIRE_ASKING_PRICE_EXP_CCN_TR_ID,
    InquireAskingPriceExpCcnRequest, InquireAskingPriceExpCcnResponse,
};
pub use inquire_ccnl::{
    INQUIRE_CCNL_PATH, INQUIRE_CCNL_TR_ID, InquireCcnlRequest, InquireCcnlResponse,
};
pub use inquire_daily_item_chart_price::{
    INQUIRE_DAILY_ITEM_CHART_PRICE_PATH, INQUIRE_DAILY_ITEM_CHART_PRICE_TR_ID,
    InquireDailyItemChartPriceRequest, InquireDailyItemChartPriceResponse,
};
pub use inquire_daily_price::{
    INQUIRE_DAILY_PRICE_PATH, INQUIRE_DAILY_PRICE_TR_ID, InquireDailyPriceRequest,
    InquireDailyPriceResponse,
};
pub use inquire_price::{
    INQUIRE_PRICE_PATH, INQUIRE_PRICE_TR_ID, InquirePriceRequest, InquirePriceResponse,
};
pub use inquire_time_daily_chart_price::{
    INQUIRE_TIME_DAILY_CHART_PRICE_PATH, INQUIRE_TIME_DAILY_CHART_PRICE_TR_ID,
    InquireTimeDailyChartPriceRequest, InquireTimeDailyChartPriceResponse,
};
pub use inquire_time_item_chart_price::{
    INQUIRE_TIME_ITEM_CHART_PRICE_PATH, INQUIRE_TIME_ITEM_CHART_PRICE_TR_ID,
    InquireTimeItemChartPriceRequest, InquireTimeItemChartPriceResponse,
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

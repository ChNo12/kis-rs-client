mod common;
pub mod market_analysis;
pub mod quotations;
pub mod ranking;
pub mod trading;

pub use common::{Continuation, MarketDivision, StockCode};
pub use market_analysis::{
    CAPTURE_UP_LOW_PRICE_PATH, CAPTURE_UP_LOW_PRICE_TR_ID, CaptureUpLowPriceRequest,
    CaptureUpLowPriceResponse,
};
pub use quotations::{
    DoubleOutputResponse, InquireAskingPriceExpCcnRequest, InquireAskingPriceExpCcnResponse,
    InquireCcnlRequest, InquireCcnlResponse, InquireDailyItemChartPriceRequest,
    InquireDailyItemChartPriceResponse, InquireDailyPriceRequest, InquireDailyPriceResponse,
    InquirePriceRequest, InquirePriceResponse, InquireTimeDailyChartPriceRequest,
    InquireTimeDailyChartPriceResponse, InquireTimeItemChartPriceRequest,
    InquireTimeItemChartPriceResponse, SingleOutputResponse,
};
pub use ranking::{
    AFTER_HOUR_BALANCE_PATH, AFTER_HOUR_BALANCE_TR_ID, AfterHourBalanceRequest,
    AfterHourBalanceResponse, BULK_TRANS_NUM_PATH, BULK_TRANS_NUM_TR_ID, BulkTransNumRequest,
    BulkTransNumResponse, FLUCTUATION_PATH, FLUCTUATION_TR_ID, FluctuationRequest,
    FluctuationResponse,
};
pub use trading::{
    AllQuantityOrder, InquireDailyCcldPeriod, InquireDailyCcldRequest, InquireDailyCcldResponse,
    InquirePsblRvsecnclRequest, InquirePsblRvsecnclResponse, OrderCashRequest, OrderCashResponse,
    OrderRvsecnclRequest, OrderRvsecnclResponse, OrderSide, ReviseCancel,
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

    pub fn quotations(self) -> quotations::Service<'a, T> {
        quotations::Service::new(self.client)
    }

    pub fn ranking(self) -> ranking::Service<'a, T> {
        ranking::Service::new(self.client)
    }

    pub fn market_analysis(self) -> market_analysis::Service<'a, T> {
        market_analysis::Service::new(self.client)
    }

    pub fn trading(self) -> trading::Service<'a, T> {
        trading::Service::new(self.client)
    }
}

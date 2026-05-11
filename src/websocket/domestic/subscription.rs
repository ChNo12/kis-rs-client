use crate::config::Environment;
use crate::error::Result;
use crate::websocket::subscription::{Subscription, SubscriptionAction};

pub const DOMESTIC_REALTIME_PRICE_KRX_TR_ID: &str = "H0STCNT0";
pub const DOMESTIC_REALTIME_PRICE_NXT_TR_ID: &str = "H0NXCNT0";
pub const DOMESTIC_REALTIME_PRICE_UNIFIED_TR_ID: &str = "H0UNCNT0";
pub const DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID: &str = "H0STCNI0";
pub const DOMESTIC_EXECUTION_NOTICE_MOCK_TR_ID: &str = "H0STCNI9";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomesticRealtimePriceMarket {
    Krx,
    Nxt,
    Unified,
}

impl DomesticRealtimePriceMarket {
    pub const fn tr_id(self) -> &'static str {
        match self {
            Self::Krx => DOMESTIC_REALTIME_PRICE_KRX_TR_ID,
            Self::Nxt => DOMESTIC_REALTIME_PRICE_NXT_TR_ID,
            Self::Unified => DOMESTIC_REALTIME_PRICE_UNIFIED_TR_ID,
        }
    }
}

pub fn realtime_price_subscription(
    action: SubscriptionAction,
    market: DomesticRealtimePriceMarket,
    stock_code: impl Into<String>,
) -> Result<Subscription> {
    Subscription::new(action, market.tr_id(), stock_code)
}

pub fn execution_notice_subscription(
    action: SubscriptionAction,
    environment: Environment,
    hts_id: impl Into<String>,
) -> Result<Subscription> {
    let tr_id = match environment {
        Environment::Real => DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID,
        Environment::Mock => DOMESTIC_EXECUTION_NOTICE_MOCK_TR_ID,
    };

    Subscription::new(action, tr_id, hts_id)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use super::*;

    #[test]
    fn realtime_price_subscription_uses_market_tr_id() {
        let subscription = realtime_price_subscription(
            SubscriptionAction::Subscribe,
            DomesticRealtimePriceMarket::Krx,
            "005930",
        )
        .unwrap();

        assert_eq!(subscription.tr_id, DOMESTIC_REALTIME_PRICE_KRX_TR_ID);
        assert_eq!(subscription.tr_key, "005930");
    }

    #[test]
    fn execution_notice_subscription_uses_environment_specific_tr_id() {
        let subscription = execution_notice_subscription(
            SubscriptionAction::Subscribe,
            Environment::Mock,
            "hts-id",
        )
        .unwrap();

        assert_eq!(subscription.tr_id, DOMESTIC_EXECUTION_NOTICE_MOCK_TR_ID);

        let subscription = execution_notice_subscription(
            SubscriptionAction::Subscribe,
            Environment::Real,
            "hts-id",
        )
        .unwrap();

        assert_eq!(subscription.tr_id, DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID);
    }

    #[test]
    fn subscription_rejects_empty_tr_key() {
        assert_eq!(
            execution_notice_subscription(SubscriptionAction::Subscribe, Environment::Mock, ""),
            Err(Error::config("websocket tr_key is empty"))
        );
    }
}

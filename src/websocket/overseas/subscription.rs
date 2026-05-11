use crate::config::Environment;
use crate::error::Result;
use crate::websocket::subscription::{Subscription, SubscriptionAction};

pub const OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID: &str = "H0GSCNI0";
pub const OVERSEAS_EXECUTION_NOTICE_MOCK_TR_ID: &str = "H0GSCNI9";

pub fn execution_notice_subscription(
    action: SubscriptionAction,
    environment: Environment,
    hts_id: impl Into<String>,
) -> Result<Subscription> {
    let tr_id = match environment {
        Environment::Real => OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID,
        Environment::Mock => OVERSEAS_EXECUTION_NOTICE_MOCK_TR_ID,
    };

    Subscription::new(action, tr_id, hts_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_notice_subscription_uses_environment_specific_tr_id() {
        let subscription = execution_notice_subscription(
            SubscriptionAction::Subscribe,
            Environment::Mock,
            "hts-id",
        )
        .unwrap();

        assert_eq!(subscription.tr_id, OVERSEAS_EXECUTION_NOTICE_MOCK_TR_ID);

        let subscription = execution_notice_subscription(
            SubscriptionAction::Subscribe,
            Environment::Real,
            "hts-id",
        )
        .unwrap();

        assert_eq!(subscription.tr_id, OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID);
    }
}

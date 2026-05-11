use std::fmt;

use serde::Serialize;

use crate::auth::ApprovalKey;
use crate::error::{Error, Result};

pub const CONTENT_TYPE_UTF8: &str = "utf-8";
pub const PERSONAL_CUSTOMER_TYPE: &str = "P";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubscriptionAction {
    Subscribe,
    Unsubscribe,
}

impl SubscriptionAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Subscribe => "1",
            Self::Unsubscribe => "2",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub action: SubscriptionAction,
    pub tr_id: String,
    pub tr_key: String,
}

impl Subscription {
    pub fn new(
        action: SubscriptionAction,
        tr_id: impl Into<String>,
        tr_key: impl Into<String>,
    ) -> Result<Self> {
        let tr_id = tr_id.into();
        let tr_key = tr_key.into();

        if tr_id.is_empty() {
            return Err(Error::config("websocket tr_id is empty"));
        }

        if tr_key.is_empty() {
            return Err(Error::config("websocket tr_key is empty"));
        }

        Ok(Self {
            action,
            tr_id,
            tr_key,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SubscriptionBook {
    subscriptions: Vec<Subscription>,
}

impl SubscriptionBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }

    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }

    pub fn subscriptions(&self) -> &[Subscription] {
        &self.subscriptions
    }

    pub fn messages(&self, approval_key: &ApprovalKey) -> Vec<SubscriptionMessage> {
        self.subscriptions
            .iter()
            .cloned()
            .map(|subscription| SubscriptionMessage::new(approval_key, subscription))
            .collect()
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct SubscriptionMessage {
    pub header: SubscriptionMessageHeader,
    pub body: SubscriptionMessageBody,
}

impl SubscriptionMessage {
    pub fn new(approval_key: &ApprovalKey, subscription: Subscription) -> Self {
        Self {
            header: SubscriptionMessageHeader {
                content_type: CONTENT_TYPE_UTF8.to_string(),
                approval_key: approval_key.expose_secret().to_string(),
                tr_type: subscription.action.as_str().to_string(),
                custtype: PERSONAL_CUSTOMER_TYPE.to_string(),
            },
            body: SubscriptionMessageBody {
                input: SubscriptionMessageInput {
                    tr_id: subscription.tr_id,
                    tr_key: subscription.tr_key,
                },
            },
        }
    }
}

impl fmt::Debug for SubscriptionMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubscriptionMessage")
            .field("header", &self.header)
            .field("body", &self.body)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct SubscriptionMessageHeader {
    #[serde(rename = "content-type")]
    pub content_type: String,
    pub approval_key: String,
    pub tr_type: String,
    pub custtype: String,
}

impl fmt::Debug for SubscriptionMessageHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubscriptionMessageHeader")
            .field("content_type", &self.content_type)
            .field("approval_key", &"***")
            .field("tr_type", &self.tr_type)
            .field("custtype", &self.custtype)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubscriptionMessageBody {
    pub input: SubscriptionMessageInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubscriptionMessageInput {
    pub tr_id: String,
    pub tr_key: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn subscription_message_serializes_request() {
        let approval_key = ApprovalKey::new("approval-key-value");
        let subscription =
            Subscription::new(SubscriptionAction::Subscribe, "H0STCNT0", "005930").unwrap();

        let message = SubscriptionMessage::new(&approval_key, subscription);

        let value = serde_json::to_value(&message).unwrap();
        assert_eq!(
            value,
            json!({
                "header": {
                    "content-type": "utf-8",
                    "approval_key": "approval-key-value",
                    "tr_type": "1",
                    "custtype": "P"
                },
                "body": {
                    "input": {
                        "tr_id": "H0STCNT0",
                        "tr_key": "005930"
                    }
                }
            })
        );

        let debug = format!("{message:?}");
        assert!(!debug.contains("approval-key-value"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn subscription_rejects_empty_tr_id() {
        assert_eq!(
            Subscription::new(SubscriptionAction::Subscribe, "", "005930"),
            Err(Error::config("websocket tr_id is empty"))
        );
    }

    #[test]
    fn subscription_rejects_empty_tr_key() {
        assert_eq!(
            Subscription::new(SubscriptionAction::Subscribe, "H0STCNT0", ""),
            Err(Error::config("websocket tr_key is empty"))
        );
    }

    #[test]
    fn subscription_book_builds_messages_for_resubscribe() {
        let approval_key = ApprovalKey::new("approval-key-value");
        let mut book = SubscriptionBook::new();
        book.add(Subscription::new(SubscriptionAction::Subscribe, "H0STCNT0", "005930").unwrap());
        book.add(Subscription::new(SubscriptionAction::Subscribe, "H0GSCNI9", "hts-id").unwrap());

        let messages = book.messages(&approval_key);

        assert_eq!(book.len(), 2);
        assert_eq!(messages[0].body.input.tr_id, "H0STCNT0");
        assert_eq!(messages[1].body.input.tr_id, "H0GSCNI9");
    }
}

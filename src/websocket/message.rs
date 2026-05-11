use serde::Deserialize;

use crate::error::{Error, Result};
use crate::websocket::domestic::{
    DOMESTIC_EXECUTION_NOTICE_MOCK_TR_ID, DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID,
    DOMESTIC_REALTIME_PRICE_KRX_TR_ID, DOMESTIC_REALTIME_PRICE_NXT_TR_ID,
    DOMESTIC_REALTIME_PRICE_UNIFIED_TR_ID,
};
use crate::websocket::overseas::{
    OVERSEAS_EXECUTION_NOTICE_MOCK_TR_ID, OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID,
};

pub const PINGPONG_TR_ID: &str = "PINGPONG";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingFrame {
    Data(RealtimeDataFrame),
    System(SystemMessage),
}

impl IncomingFrame {
    pub fn parse(raw: &str) -> Result<Self> {
        if raw.starts_with('0') || raw.starts_with('1') {
            return RealtimeDataFrame::parse(raw).map(Self::Data);
        }

        SystemMessage::parse(raw).map(Self::System)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealtimeDataFrame {
    pub tr_type: String,
    pub tr_id: String,
    pub record_count: String,
    pub payload: String,
}

impl RealtimeDataFrame {
    pub fn parse(raw: &str) -> Result<Self> {
        let mut parts = raw.splitn(4, '|');
        let tr_type = parts.next().unwrap_or_default();
        let tr_id = parts.next();
        let record_count = parts.next();
        let payload = parts.next();

        match (tr_id, record_count, payload) {
            (Some(tr_id), Some(record_count), Some(payload)) => Ok(Self {
                tr_type: tr_type.to_string(),
                tr_id: tr_id.to_string(),
                record_count: record_count.to_string(),
                payload: payload.to_string(),
            }),
            _ => Err(Error::parse("websocket data frame is malformed")),
        }
    }

    pub fn is_domestic_execution_notice(&self) -> bool {
        matches!(
            self.tr_id.as_str(),
            DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID | DOMESTIC_EXECUTION_NOTICE_MOCK_TR_ID
        )
    }

    pub fn is_domestic_realtime_price(&self) -> bool {
        matches!(
            self.tr_id.as_str(),
            DOMESTIC_REALTIME_PRICE_KRX_TR_ID
                | DOMESTIC_REALTIME_PRICE_NXT_TR_ID
                | DOMESTIC_REALTIME_PRICE_UNIFIED_TR_ID
        )
    }

    pub fn is_overseas_execution_notice(&self) -> bool {
        matches!(
            self.tr_id.as_str(),
            OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID | OVERSEAS_EXECUTION_NOTICE_MOCK_TR_ID
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SystemMessage {
    pub header: SystemMessageHeader,
    pub body: Option<SystemMessageBody>,
}

impl SystemMessage {
    pub fn parse(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|error| {
            Error::parse(format!("failed to parse websocket system message: {error}"))
        })
    }

    pub fn is_ping_pong(&self) -> bool {
        self.header.tr_id == PINGPONG_TR_ID
    }

    pub fn is_success(&self) -> bool {
        self.body.as_ref().and_then(|body| body.rt_cd.as_deref()) == Some("0")
    }

    pub fn encryption_key(&self) -> Option<&str> {
        self.body
            .as_ref()
            .and_then(|body| body.output.as_ref())
            .and_then(|output| output.key.as_deref())
    }

    pub fn encryption_iv(&self) -> Option<&str> {
        self.body
            .as_ref()
            .and_then(|body| body.output.as_ref())
            .and_then(|output| output.iv.as_deref())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SystemMessageHeader {
    pub tr_id: String,
    pub tr_key: Option<String>,
    pub encrypt: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SystemMessageBody {
    pub rt_cd: Option<String>,
    pub msg_cd: Option<String>,
    pub msg1: Option<String>,
    pub output: Option<SystemMessageOutput>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SystemMessageOutput {
    pub iv: Option<String>,
    pub key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_realtime_data_frame() {
        let frame = IncomingFrame::parse("0|H0STCNT0|001|005930^093000^70500").unwrap();

        assert_eq!(
            frame,
            IncomingFrame::Data(RealtimeDataFrame {
                tr_type: "0".to_string(),
                tr_id: "H0STCNT0".to_string(),
                record_count: "001".to_string(),
                payload: "005930^093000^70500".to_string(),
            })
        );
    }

    #[test]
    fn rejects_malformed_realtime_data_frame() {
        assert!(matches!(
            IncomingFrame::parse("0|H0STCNT0"),
            Err(Error::Parse { .. })
        ));
    }

    #[test]
    fn parses_pingpong_system_message() {
        let frame = IncomingFrame::parse(
            r#"{
                "header": {
                    "tr_id": "PINGPONG"
                }
            }"#,
        )
        .unwrap();

        match frame {
            IncomingFrame::System(message) => assert!(message.is_ping_pong()),
            IncomingFrame::Data(_) => panic!("expected system message"),
        }
    }

    #[test]
    fn parses_subscription_system_message_with_encryption_keys() {
        let message = SystemMessage::parse(
            r#"{
                "header": {
                    "tr_id": "H0STCNI0",
                    "tr_key": "hts-id",
                    "encrypt": "Y"
                },
                "body": {
                    "rt_cd": "0",
                    "msg_cd": "OPSP0000",
                    "msg1": "SUBSCRIBE SUCCESS",
                    "output": {
                        "iv": "iv-value",
                        "key": "key-value"
                    }
                }
            }"#,
        )
        .unwrap();

        assert!(message.is_success());
        assert_eq!(message.encryption_iv(), Some("iv-value"));
        assert_eq!(message.encryption_key(), Some("key-value"));
    }
}

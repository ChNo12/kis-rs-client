use std::fmt;

use rust_decimal::Decimal;

use crate::error::Result;
use crate::websocket::util::{CaretFields, mask_tail};

#[derive(Clone, Eq, PartialEq)]
pub struct OverseasExecutionNotice {
    pub customer_id: String,
    pub account_no: String,
    pub order_no: String,
    pub original_order_no: String,
    pub sell_buy_class: String,
    pub receipt_class: String,
    pub order_kind: String,
    pub stock_code: String,
    pub conclusion_quantity: Option<Decimal>,
    pub conclusion_unit_price: Option<Decimal>,
    pub stock_conclusion_time: String,
    pub refused: String,
    pub concluded: String,
    pub accepted: String,
    pub branch_no: String,
    pub order_quantity: Option<Decimal>,
    pub account_name: String,
    pub conclusion_name: String,
    pub order_condition: String,
    pub debt_class: String,
    pub debt_date: String,
    pub start_time: String,
    pub end_time: String,
    pub time_division_type: String,
    pub conclusion_unit_price12: Option<Decimal>,
}

impl OverseasExecutionNotice {
    pub const MIN_FIELD_COUNT: usize = 21;
    pub const MAX_FIELD_COUNT: usize = 25;

    pub fn parse(payload: &str) -> Result<Self> {
        let fields = CaretFields::new_range(
            payload,
            Self::MIN_FIELD_COUNT,
            Self::MAX_FIELD_COUNT,
            "overseas execution notice",
        )?;

        Ok(Self {
            customer_id: fields.text(0),
            account_no: fields.text(1),
            order_no: fields.text(2),
            original_order_no: fields.text(3),
            sell_buy_class: fields.text(4),
            receipt_class: fields.text(5),
            order_kind: fields.text(6),
            stock_code: fields.text(7),
            conclusion_quantity: fields
                .optional_decimal(8, "overseas execution notice conclusion quantity")?,
            conclusion_unit_price: fields
                .optional_decimal(9, "overseas execution notice conclusion unit price")?,
            stock_conclusion_time: fields.text(10),
            refused: fields.text(11),
            concluded: fields.text(12),
            accepted: fields.text(13),
            branch_no: fields.text(14),
            order_quantity: fields
                .optional_decimal(15, "overseas execution notice order quantity")?,
            account_name: fields.text(16),
            conclusion_name: fields.text(17),
            order_condition: fields.text(18),
            debt_class: fields.text(19),
            debt_date: fields.text(20),
            start_time: fields.optional_text(21),
            end_time: fields.optional_text(22),
            time_division_type: fields.optional_text(23),
            conclusion_unit_price12: fields
                .optional_decimal_at(24, "overseas execution notice conclusion unit price12")?,
        })
    }
}

impl fmt::Debug for OverseasExecutionNotice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverseasExecutionNotice")
            .field("customer_id", &mask_tail(&self.customer_id, 2))
            .field("account_no", &mask_tail(&self.account_no, 2))
            .field("order_no", &self.order_no)
            .field("original_order_no", &self.original_order_no)
            .field("sell_buy_class", &self.sell_buy_class)
            .field("receipt_class", &self.receipt_class)
            .field("order_kind", &self.order_kind)
            .field("stock_code", &self.stock_code)
            .field("conclusion_quantity", &self.conclusion_quantity)
            .field("conclusion_unit_price", &self.conclusion_unit_price)
            .field("stock_conclusion_time", &self.stock_conclusion_time)
            .field("refused", &self.refused)
            .field("concluded", &self.concluded)
            .field("accepted", &self.accepted)
            .field("branch_no", &self.branch_no)
            .field("order_quantity", &self.order_quantity)
            .field("account_name", &mask_tail(&self.account_name, 2))
            .field("conclusion_name", &self.conclusion_name)
            .field("order_condition", &self.order_condition)
            .field("debt_class", &self.debt_class)
            .field("debt_date", &self.debt_date)
            .field("start_time", &self.start_time)
            .field("end_time", &self.end_time)
            .field("time_division_type", &self.time_division_type)
            .field("conclusion_unit_price12", &self.conclusion_unit_price12)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[cfg(feature = "websocket-client")]
    use crate::websocket::ExecutionNoticeCipher;

    #[cfg(feature = "websocket-client")]
    #[test]
    fn decrypts_overseas_execution_notice_fixture() {
        let cipher =
            ExecutionNoticeCipher::new("0123456789abcdef0123456789abcdef", "abcdef9876543210")
                .unwrap();
        let encrypted = "MAWN08Q42FR1k/rxidypO00IuMWX0o3kXvg3oCC9igiohsPAPEsrs2Lyl+JGBl8K6l5KmV2ewSN/zH46e9iXNs3F5TDda1IOonk3/1X2gju6C2FdHSSUQ63TT40Qpw5wBpHppGni5IrhYtkGkmba47g69cMsX00meF2O+fXJ7fE=";

        let decrypted = cipher.decrypt_base64(encrypted).unwrap();
        let notice = OverseasExecutionNotice::parse(&decrypted).unwrap();

        assert_eq!(notice.customer_id, "cust");
        assert_eq!(notice.account_no, "12345678");
        assert_eq!(notice.order_no, "0001");
        assert_eq!(notice.stock_code, "AAPL");
        assert_eq!(notice.conclusion_quantity, Some(Decimal::new(10, 0)));
        assert_eq!(notice.conclusion_unit_price, Some(Decimal::new(14525, 2)));
        assert_eq!(notice.conclusion_unit_price12, Some(Decimal::new(14525, 2)));

        let debug = format!("{notice:?}");
        assert!(!debug.contains("12345678"));
        assert!(!debug.contains("account_name: \"name\""));
        assert!(debug.contains("******78"));
        assert!(debug.contains("account_name: \"**me\""));
    }

    #[test]
    fn overseas_execution_notice_rejects_invalid_decimal() {
        let payload = "cust^12345678^0001^0000^02^01^00^AAPL^not-number^145.25^093000^N^2^Y^001^10^name^AAPL INC^00^^20260511^090000^153000^0^145.25";

        assert!(matches!(
            OverseasExecutionNotice::parse(payload),
            Err(Error::Parse { .. })
        ));
    }

    #[test]
    fn overseas_execution_notice_accepts_legacy_21_field_payload() {
        let payload = "cust^12345678^0001^0000^02^01^00^AAPL^10^145.25^093000^N^2^Y^001^10^name^AAPL INC^00^^20260511";

        let notice = OverseasExecutionNotice::parse(payload).unwrap();

        assert_eq!(notice.customer_id, "cust");
        assert_eq!(notice.account_no, "12345678");
        assert_eq!(notice.stock_code, "AAPL");
        assert_eq!(notice.conclusion_quantity, Some(Decimal::new(10, 0)));
        assert_eq!(notice.conclusion_unit_price, Some(Decimal::new(14525, 2)));
        assert_eq!(notice.debt_date, "20260511");
        assert_eq!(notice.start_time, "");
        assert_eq!(notice.end_time, "");
        assert_eq!(notice.time_division_type, "");
        assert_eq!(notice.conclusion_unit_price12, None);
    }
}

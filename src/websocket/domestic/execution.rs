use std::fmt;

use rust_decimal::Decimal;

use crate::error::Result;
use crate::websocket::util::{
    CaretFields, mask_tail, normalize_kis_order_no, zero_pad_numeric_text,
};

const SELL_BUY_CLASS_LEN: usize = 2;
const ORDER_KIND_LEN: usize = 2;
const STOCK_CONCLUSION_TIME_LEN: usize = 6;
const BRANCH_NO_LEN: usize = 5;
const CREDIT_CLASS_LEN: usize = 2;

#[derive(Clone, Eq, PartialEq)]
pub struct DomesticExecutionNotice {
    pub customer_id: String,
    pub account_no: String,
    pub order_no: String,
    pub original_order_no: String,
    pub sell_buy_class: String,
    pub receipt_class: String,
    pub order_kind: String,
    pub order_condition: String,
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
    pub order_condition_price: Option<Decimal>,
    pub order_exchange: String,
    pub popup: String,
    pub filler: String,
    pub credit_class: String,
    pub credit_loan_date: String,
    pub conclusion_name: String,
    pub order_price: Option<Decimal>,
}

impl DomesticExecutionNotice {
    pub const MIN_FIELD_COUNT: usize = 23;
    pub const MAX_FIELD_COUNT: usize = 26;

    pub fn parse(payload: &str) -> Result<Self> {
        let fields = CaretFields::new_range(
            payload,
            Self::MIN_FIELD_COUNT,
            Self::MAX_FIELD_COUNT,
            "domestic execution notice",
        )?;
        let tail = DomesticExecutionNoticeTail::parse(&fields);

        Ok(Self {
            customer_id: fields.text(0),
            account_no: fields.text(1),
            order_no: normalize_kis_order_no(&fields.text(2)),
            original_order_no: normalize_kis_order_no(&fields.text(3)),
            sell_buy_class: zero_pad_numeric_text(&fields.text(4), SELL_BUY_CLASS_LEN),
            receipt_class: fields.text(5),
            order_kind: zero_pad_numeric_text(&fields.text(6), ORDER_KIND_LEN),
            order_condition: fields.text(7),
            stock_code: fields.text(8),
            conclusion_quantity: fields
                .optional_decimal(9, "domestic execution notice conclusion quantity")?,
            conclusion_unit_price: fields
                .optional_decimal(10, "domestic execution notice conclusion unit price")?,
            stock_conclusion_time: zero_pad_numeric_text(
                &fields.text(11),
                STOCK_CONCLUSION_TIME_LEN,
            ),
            refused: fields.text(12),
            concluded: fields.text(13),
            accepted: fields.text(14),
            branch_no: zero_pad_numeric_text(&fields.text(15), BRANCH_NO_LEN),
            order_quantity: fields
                .optional_decimal(16, "domestic execution notice order quantity")?,
            account_name: fields.text(17),
            order_condition_price: tail.order_condition_price,
            order_exchange: tail.order_exchange,
            popup: tail.popup,
            filler: tail.filler,
            credit_class: zero_pad_numeric_text(&tail.credit_class, CREDIT_CLASS_LEN),
            credit_loan_date: tail.credit_loan_date,
            conclusion_name: tail.conclusion_name,
            order_price: tail.order_price,
        })
    }
}

struct DomesticExecutionNoticeTail {
    order_condition_price: Option<Decimal>,
    order_exchange: String,
    popup: String,
    filler: String,
    credit_class: String,
    credit_loan_date: String,
    conclusion_name: String,
    order_price: Option<Decimal>,
}

impl DomesticExecutionNoticeTail {
    fn parse(fields: &CaretFields<'_>) -> Self {
        if fields.len() == DomesticExecutionNotice::MIN_FIELD_COUNT
            && is_compacted_virtual_tail(fields)
        {
            let compacted_order = fields.text(18);
            let mut chars = compacted_order.chars();
            let order_exchange = chars.next().map(String::from).unwrap_or_default();
            let popup = chars.collect::<String>();

            return Self {
                order_condition_price: None,
                order_exchange,
                popup,
                filler: String::new(),
                credit_class: fields.text(19),
                credit_loan_date: fields.text(20),
                conclusion_name: fields.text(21),
                order_price: lenient_decimal(fields.get(22)),
            };
        }

        Self {
            order_condition_price: fields
                .lenient_optional_decimal(18, "domestic execution notice order condition price"),
            order_exchange: fields.get(19).map(str::to_string).unwrap_or_default(),
            popup: fields.get(20).map(str::to_string).unwrap_or_default(),
            filler: fields.get(21).map(str::to_string).unwrap_or_default(),
            credit_class: fields.get(22).map(str::to_string).unwrap_or_default(),
            credit_loan_date: fields.get(23).map(str::to_string).unwrap_or_default(),
            conclusion_name: fields.get(24).map(str::to_string).unwrap_or_default(),
            order_price: lenient_decimal(fields.get(25)),
        }
    }
}

fn is_compacted_virtual_tail(fields: &CaretFields<'_>) -> bool {
    let Some(order_exchange_and_popup) = fields.get(18) else {
        return false;
    };
    let Some(credit_class) = fields.get(19) else {
        return false;
    };

    order_exchange_and_popup.chars().count() == 2
        && order_exchange_and_popup
            .chars()
            .all(|char| char.is_ascii_alphanumeric())
        && credit_class.chars().all(|char| char.is_ascii_digit())
}

fn lenient_decimal(value: Option<&str>) -> Option<Decimal> {
    value
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse().ok())
}

impl fmt::Debug for DomesticExecutionNotice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomesticExecutionNotice")
            .field("customer_id", &mask_tail(&self.customer_id, 2))
            .field("account_no", &mask_tail(&self.account_no, 2))
            .field("order_no", &self.order_no)
            .field("original_order_no", &self.original_order_no)
            .field("sell_buy_class", &self.sell_buy_class)
            .field("receipt_class", &self.receipt_class)
            .field("order_kind", &self.order_kind)
            .field("order_condition", &self.order_condition)
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
            .field("order_condition_price", &self.order_condition_price)
            .field("order_exchange", &self.order_exchange)
            .field("popup", &self.popup)
            .field("filler", &self.filler)
            .field("credit_class", &self.credit_class)
            .field("credit_loan_date", &self.credit_loan_date)
            .field("conclusion_name", &self.conclusion_name)
            .field("order_price", &self.order_price)
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
    fn decrypts_domestic_execution_notice_fixture() {
        let cipher =
            ExecutionNoticeCipher::new("0123456789abcdef0123456789abcdef", "abcdef9876543210")
                .unwrap();
        let encrypted = "MAWN08Q42FR1k/rxidypO00IuMWX0o3kXvg3oCC9igifAog6mZPi3ekVGNTMDhdiZXgw0n5uIBWb+tD+AFSui1kHuVJEZq3yXI3n2aVmC6HZ1jWZcCYizfI9jw9BaHK6Tabviif9AffWyVI9Z/djdk6gyapIULEBOr/WKKJTmmY=";

        let decrypted = cipher.decrypt_base64(encrypted).unwrap();
        let notice = DomesticExecutionNotice::parse(&decrypted).unwrap();

        assert_eq!(notice.customer_id, "cust");
        assert_eq!(notice.account_no, "12345678");
        assert_eq!(notice.order_no, "0000000001");
        assert_eq!(notice.stock_code, "005930");
        assert_eq!(notice.conclusion_quantity, Some(Decimal::new(10, 0)));
        assert_eq!(notice.conclusion_unit_price, Some(Decimal::new(70000, 0)));
        assert_eq!(notice.concluded, "2");

        let debug = format!("{notice:?}");
        assert!(!debug.contains("12345678"));
        assert!(!debug.contains("account_name: \"name\""));
        assert!(debug.contains("******78"));
        assert!(debug.contains("account_name: \"**me\""));
    }

    #[test]
    fn domestic_execution_notice_rejects_field_count_mismatch() {
        assert!(matches!(
            DomesticExecutionNotice::parse("cust^12345678"),
            Err(Error::Parse { .. })
        ));
    }

    #[test]
    fn domestic_execution_notice_accepts_virtual_23_field_payload() {
        let payload =
            "cust^12345678^34564^0000^1^0^0^0^005930^10^70000^93000^N^2^Y^1^10^name^^1^N^^1";

        let notice = DomesticExecutionNotice::parse(payload).unwrap();

        assert_eq!(notice.customer_id, "cust");
        assert_eq!(notice.account_no, "12345678");
        assert_eq!(notice.order_no, "0000034564");
        assert_eq!(notice.stock_code, "005930");
        assert_eq!(notice.conclusion_quantity, Some(Decimal::new(10, 0)));
        assert_eq!(notice.conclusion_unit_price, Some(Decimal::new(70000, 0)));
        assert_eq!(notice.credit_class, "01");
        assert_eq!(notice.credit_loan_date, "");
        assert_eq!(notice.conclusion_name, "");
        assert_eq!(notice.order_price, None);
    }

    #[test]
    fn domestic_execution_notice_parses_compacted_virtual_tail() {
        let payload = "cust^12345678^17214^^02^0^00^0^472150^1^23000^104556^0^1^1^00950^1^name^1Y^10^^TIGER 배당커버드콜액티브^";

        let notice = DomesticExecutionNotice::parse(payload).unwrap();

        assert_eq!(notice.order_no, "0000017214");
        assert_eq!(notice.original_order_no, "");
        assert_eq!(notice.stock_code, "472150");
        assert_eq!(notice.order_condition_price, None);
        assert_eq!(notice.order_exchange, "1");
        assert_eq!(notice.popup, "Y");
        assert_eq!(notice.filler, "");
        assert_eq!(notice.credit_class, "10");
        assert_eq!(notice.credit_loan_date, "");
        assert_eq!(notice.conclusion_name, "TIGER 배당커버드콜액티브");
        assert_eq!(notice.order_price, None);
    }

    #[test]
    fn domestic_execution_notice_normalizes_fixed_width_numeric_fields() {
        let payload = "cust^12345678^34564^0000^1^0^0^0^005930^10^70000^93000^N^2^Y^1^10^name^^1^N^^1^20260511^SAMSUNG^70000";

        let notice = DomesticExecutionNotice::parse(payload).unwrap();

        assert_eq!(notice.order_no, "0000034564");
        assert_eq!(notice.original_order_no, "0000");
        assert_eq!(notice.sell_buy_class, "01");
        assert_eq!(notice.order_kind, "00");
        assert_eq!(notice.stock_conclusion_time, "093000");
        assert_eq!(notice.branch_no, "00001");
        assert_eq!(notice.credit_class, "01");
    }
}

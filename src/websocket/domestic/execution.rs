use std::fmt;

use rust_decimal::Decimal;

use crate::error::Result;
use crate::websocket::util::{CaretFields, mask_tail};

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
    pub const FIELD_COUNT: usize = 26;

    pub fn parse(payload: &str) -> Result<Self> {
        let fields = CaretFields::new(payload, Self::FIELD_COUNT, "domestic execution notice")?;

        Ok(Self {
            customer_id: fields.text(0),
            account_no: fields.text(1),
            order_no: fields.text(2),
            original_order_no: fields.text(3),
            sell_buy_class: fields.text(4),
            receipt_class: fields.text(5),
            order_kind: fields.text(6),
            order_condition: fields.text(7),
            stock_code: fields.text(8),
            conclusion_quantity: fields
                .optional_decimal(9, "domestic execution notice conclusion quantity")?,
            conclusion_unit_price: fields
                .optional_decimal(10, "domestic execution notice conclusion unit price")?,
            stock_conclusion_time: fields.text(11),
            refused: fields.text(12),
            concluded: fields.text(13),
            accepted: fields.text(14),
            branch_no: fields.text(15),
            order_quantity: fields
                .optional_decimal(16, "domestic execution notice order quantity")?,
            account_name: fields.text(17),
            order_condition_price: fields
                .optional_decimal(18, "domestic execution notice order condition price")?,
            order_exchange: fields.text(19),
            popup: fields.text(20),
            filler: fields.text(21),
            credit_class: fields.text(22),
            credit_loan_date: fields.text(23),
            conclusion_name: fields.text(24),
            order_price: fields.optional_decimal(25, "domestic execution notice order price")?,
        })
    }
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
        assert_eq!(notice.order_no, "0001");
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
}

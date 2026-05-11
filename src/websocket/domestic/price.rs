use rust_decimal::Decimal;

use crate::error::Result;
use crate::websocket::RealtimeDataFrame;
use crate::websocket::util::CaretFields;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomesticRealtimePrice {
    pub stock_code: String,
    pub stock_conclusion_time: String,
    pub current_price: Option<Decimal>,
    pub previous_day_difference_sign: String,
    pub previous_day_difference: Option<Decimal>,
    pub previous_day_rate: Option<Decimal>,
    pub weighted_average_stock_price: Option<Decimal>,
    pub open_price: Option<Decimal>,
    pub high_price: Option<Decimal>,
    pub low_price: Option<Decimal>,
    pub ask_price1: Option<Decimal>,
    pub bid_price1: Option<Decimal>,
    pub conclusion_volume: Option<Decimal>,
    pub accumulated_volume: Option<Decimal>,
    pub accumulated_trade_amount: Option<Decimal>,
    pub sell_conclusion_count: Option<Decimal>,
    pub buy_conclusion_count: Option<Decimal>,
    pub net_buy_conclusion_count: Option<Decimal>,
    pub conclusion_intensity: Option<Decimal>,
    pub total_sell_conclusion_quantity: Option<Decimal>,
    pub total_buy_conclusion_quantity: Option<Decimal>,
    pub conclusion_division: String,
    pub buy_rate: Option<Decimal>,
    pub previous_day_volume_to_accumulated_volume_rate: Option<Decimal>,
    pub open_price_time: String,
    pub open_price_to_current_price_sign: String,
    pub open_price_to_current_price: Option<Decimal>,
    pub high_price_time: String,
    pub high_price_to_current_price_sign: String,
    pub high_price_to_current_price: Option<Decimal>,
    pub low_price_time: String,
    pub low_price_to_current_price_sign: String,
    pub low_price_to_current_price: Option<Decimal>,
    pub business_date: String,
    pub new_market_open_class_code: String,
    pub trading_halt: String,
    pub ask_remain_quantity1: Option<Decimal>,
    pub bid_remain_quantity1: Option<Decimal>,
    pub total_ask_remain_quantity: Option<Decimal>,
    pub total_bid_remain_quantity: Option<Decimal>,
    pub volume_turnover_rate: Option<Decimal>,
    pub previous_same_time_accumulated_volume: Option<Decimal>,
    pub previous_same_time_accumulated_volume_rate: Option<Decimal>,
    pub hour_class_code: String,
    pub market_time_class_code: String,
    pub vi_standard_price: Option<Decimal>,
}

impl DomesticRealtimePrice {
    pub const FIELD_COUNT: usize = 46;

    pub const COLUMNS: [&'static str; Self::FIELD_COUNT] = [
        "MKSC_SHRN_ISCD",
        "STCK_CNTG_HOUR",
        "STCK_PRPR",
        "PRDY_VRSS_SIGN",
        "PRDY_VRSS",
        "PRDY_CTRT",
        "WGHN_AVRG_STCK_PRC",
        "STCK_OPRC",
        "STCK_HGPR",
        "STCK_LWPR",
        "ASKP1",
        "BIDP1",
        "CNTG_VOL",
        "ACML_VOL",
        "ACML_TR_PBMN",
        "SELN_CNTG_CSNU",
        "SHNU_CNTG_CSNU",
        "NTBY_CNTG_CSNU",
        "CTTR",
        "SELN_CNTG_SMTN",
        "SHNU_CNTG_SMTN",
        "CCLD_DVSN_OR_CNTG_CLS_CODE",
        "SHNU_RATE",
        "PRDY_VOL_VRSS_ACML_VOL_RATE",
        "OPRC_HOUR",
        "OPRC_VRSS_PRPR_SIGN",
        "OPRC_VRSS_PRPR",
        "HGPR_HOUR",
        "HGPR_VRSS_PRPR_SIGN",
        "HGPR_VRSS_PRPR",
        "LWPR_HOUR",
        "LWPR_VRSS_PRPR_SIGN",
        "LWPR_VRSS_PRPR",
        "BSOP_DATE",
        "NEW_MKOP_CLS_CODE",
        "TRHT_YN",
        "ASKP_RSQN1",
        "BIDP_RSQN1",
        "TOTAL_ASKP_RSQN",
        "TOTAL_BIDP_RSQN",
        "VOL_TNRT",
        "PRDY_SMNS_HOUR_ACML_VOL",
        "PRDY_SMNS_HOUR_ACML_VOL_RATE",
        "HOUR_CLS_CODE",
        "MRKT_TRTM_CLS_CODE",
        "VI_STND_PRC",
    ];

    pub fn parse(payload: &str) -> Result<Self> {
        let fields = CaretFields::new(payload, Self::FIELD_COUNT, "domestic realtime price")?;

        Ok(Self {
            stock_code: fields.text(0),
            stock_conclusion_time: fields.text(1),
            current_price: parse_price(&fields, 2, "current price")?,
            previous_day_difference_sign: fields.text(3),
            previous_day_difference: parse_price(&fields, 4, "previous day difference")?,
            previous_day_rate: parse_price(&fields, 5, "previous day rate")?,
            weighted_average_stock_price: parse_price(&fields, 6, "weighted average stock price")?,
            open_price: parse_price(&fields, 7, "open price")?,
            high_price: parse_price(&fields, 8, "high price")?,
            low_price: parse_price(&fields, 9, "low price")?,
            ask_price1: parse_price(&fields, 10, "ask price1")?,
            bid_price1: parse_price(&fields, 11, "bid price1")?,
            conclusion_volume: parse_price(&fields, 12, "conclusion volume")?,
            accumulated_volume: parse_price(&fields, 13, "accumulated volume")?,
            accumulated_trade_amount: parse_price(&fields, 14, "accumulated trade amount")?,
            sell_conclusion_count: parse_price(&fields, 15, "sell conclusion count")?,
            buy_conclusion_count: parse_price(&fields, 16, "buy conclusion count")?,
            net_buy_conclusion_count: parse_price(&fields, 17, "net buy conclusion count")?,
            conclusion_intensity: parse_price(&fields, 18, "conclusion intensity")?,
            total_sell_conclusion_quantity: parse_price(
                &fields,
                19,
                "total sell conclusion quantity",
            )?,
            total_buy_conclusion_quantity: parse_price(
                &fields,
                20,
                "total buy conclusion quantity",
            )?,
            conclusion_division: fields.text(21),
            buy_rate: parse_price(&fields, 22, "buy rate")?,
            previous_day_volume_to_accumulated_volume_rate: parse_price(
                &fields,
                23,
                "previous day volume to accumulated volume rate",
            )?,
            open_price_time: fields.text(24),
            open_price_to_current_price_sign: fields.text(25),
            open_price_to_current_price: parse_price(&fields, 26, "open price to current price")?,
            high_price_time: fields.text(27),
            high_price_to_current_price_sign: fields.text(28),
            high_price_to_current_price: parse_price(&fields, 29, "high price to current price")?,
            low_price_time: fields.text(30),
            low_price_to_current_price_sign: fields.text(31),
            low_price_to_current_price: parse_price(&fields, 32, "low price to current price")?,
            business_date: fields.text(33),
            new_market_open_class_code: fields.text(34),
            trading_halt: fields.text(35),
            ask_remain_quantity1: parse_price(&fields, 36, "ask remain quantity1")?,
            bid_remain_quantity1: parse_price(&fields, 37, "bid remain quantity1")?,
            total_ask_remain_quantity: parse_price(&fields, 38, "total ask remain quantity")?,
            total_bid_remain_quantity: parse_price(&fields, 39, "total bid remain quantity")?,
            volume_turnover_rate: parse_price(&fields, 40, "volume turnover rate")?,
            previous_same_time_accumulated_volume: parse_price(
                &fields,
                41,
                "previous same time accumulated volume",
            )?,
            previous_same_time_accumulated_volume_rate: parse_price(
                &fields,
                42,
                "previous same time accumulated volume rate",
            )?,
            hour_class_code: fields.text(43),
            market_time_class_code: fields.text(44),
            vi_standard_price: parse_price(&fields, 45, "vi standard price")?,
        })
    }

    pub fn from_frame(frame: &RealtimeDataFrame) -> Result<Option<Self>> {
        if frame.is_domestic_realtime_price() {
            Self::parse(&frame.payload).map(Some)
        } else {
            Ok(None)
        }
    }
}

fn parse_price(
    fields: &CaretFields<'_>,
    index: usize,
    context: &'static str,
) -> Result<Option<Decimal>> {
    fields.optional_decimal(index, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::websocket::domestic::DOMESTIC_REALTIME_PRICE_KRX_TR_ID;

    #[test]
    fn parses_domestic_realtime_price_frame() {
        let payload = "005930^093000^70500^2^1000^1.44^70400^69000^71000^68000^70600^70500^10^100000^7050000000^5^7^2^103.5^50^70^2^58.3^120.4^090000^2^1500^100000^2^500^093000^5^2500^20260511^0^N^1000^900^10000^9000^1.5^80000^125.0^0^1^72000";
        let frame = RealtimeDataFrame {
            tr_type: "0".to_string(),
            tr_id: DOMESTIC_REALTIME_PRICE_KRX_TR_ID.to_string(),
            record_count: "001".to_string(),
            payload: payload.to_string(),
        };

        assert!(frame.is_domestic_realtime_price());

        let price = DomesticRealtimePrice::from_frame(&frame).unwrap().unwrap();

        assert_eq!(price.stock_code, "005930");
        assert_eq!(price.current_price, Some(Decimal::new(70500, 0)));
        assert_eq!(price.previous_day_rate, Some(Decimal::new(144, 2)));
        assert_eq!(price.accumulated_volume, Some(Decimal::new(100000, 0)));
        assert_eq!(price.conclusion_division, "2");
        assert_eq!(price.business_date, "20260511");
        assert_eq!(price.vi_standard_price, Some(Decimal::new(72000, 0)));
    }

    #[test]
    fn domestic_realtime_price_rejects_field_count_mismatch() {
        assert!(matches!(
            DomesticRealtimePrice::parse("005930^093000^70500"),
            Err(Error::Parse { .. })
        ));
    }
}

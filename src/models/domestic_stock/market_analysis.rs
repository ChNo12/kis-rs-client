use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CaptureUpLowPriceItem {
    #[serde(rename = "mksc_shrn_iscd")]
    pub stock_code: String,
    #[serde(rename = "hts_kor_isnm")]
    pub name: String,
    #[serde(
        rename = "stck_prpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub current_price: Decimal,
    #[serde(rename = "prdy_vrss_sign")]
    pub previous_day_difference_sign: String,
    #[serde(
        rename = "prdy_vrss",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_difference: Decimal,
    #[serde(
        rename = "prdy_ctrt",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_rate: Decimal,
    #[serde(rename = "acml_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub accumulated_volume: Decimal,
    #[serde(
        rename = "total_askp_rsqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub total_ask_quantity: Decimal,
    #[serde(
        rename = "total_bidp_rsqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub total_bid_quantity: Decimal,
    #[serde(
        rename = "askp_rsqn1",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub ask_quantity1: Decimal,
    #[serde(
        rename = "bidp_rsqn1",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub bid_quantity1: Decimal,
    #[serde(rename = "prdy_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub previous_day_volume: Decimal,
    #[serde(
        rename = "seln_cnqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub sell_conclusion_quantity: Decimal,
    #[serde(
        rename = "shnu_cnqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub buy_conclusion_quantity: Decimal,
    #[serde(
        rename = "stck_llam",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub lower_limit_price: Decimal,
    #[serde(
        rename = "stck_mxpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub upper_limit_price: Decimal,
    #[serde(
        rename = "prdy_vrss_vol_rate",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_volume_rate: Decimal,
}

fn deserialize_decimal_from_str<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

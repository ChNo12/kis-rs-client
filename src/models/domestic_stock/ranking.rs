use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct AfterHourBalanceItem {
    #[serde(rename = "stck_shrn_iscd")]
    pub stock_code: String,
    #[serde(
        rename = "data_rank",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub data_rank: Decimal,
    #[serde(rename = "hts_kor_isnm")]
    pub name: String,
    #[serde(
        rename = "stck_prpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub current_price: Decimal,
    #[serde(
        rename = "prdy_vrss",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_difference: Decimal,
    #[serde(rename = "prdy_vrss_sign")]
    pub previous_day_difference_sign: String,
    #[serde(
        rename = "prdy_ctrt",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_rate: Decimal,
    #[serde(
        rename = "ovtm_total_askp_rsqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub overtime_total_ask_quantity: Decimal,
    #[serde(
        rename = "ovtm_total_bidp_rsqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub overtime_total_bid_quantity: Decimal,
    #[serde(
        rename = "mkob_otcp_vol",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub market_open_overtime_close_volume: Decimal,
    #[serde(
        rename = "mkfa_otcp_vol",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub market_finish_overtime_close_volume: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct BulkTransNumItem {
    #[serde(rename = "mksc_shrn_iscd")]
    pub stock_code: String,
    #[serde(
        rename = "data_rank",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub data_rank: Decimal,
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
        rename = "shnu_cntg_csnu",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub buy_conclusion_count: Decimal,
    #[serde(
        rename = "seln_cntg_csnu",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub sell_conclusion_count: Decimal,
    #[serde(
        rename = "ntby_cnqn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub net_buy_conclusion_quantity: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct FluctuationItem {
    #[serde(rename = "stck_shrn_iscd")]
    pub stock_code: String,
    #[serde(
        rename = "data_rank",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub data_rank: Decimal,
    #[serde(rename = "hts_kor_isnm")]
    pub name: String,
    #[serde(
        rename = "stck_prpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub current_price: Decimal,
    #[serde(
        rename = "prdy_vrss",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_difference: Decimal,
    #[serde(rename = "prdy_vrss_sign")]
    pub previous_day_difference_sign: String,
    #[serde(
        rename = "prdy_ctrt",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub previous_day_rate: Decimal,
    #[serde(rename = "acml_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub accumulated_volume: Decimal,
    #[serde(
        rename = "stck_hgpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub high_price: Decimal,
    #[serde(rename = "hgpr_hour")]
    pub high_price_time: String,
    #[serde(rename = "acml_hgpr_date")]
    pub accumulated_high_price_date: String,
    #[serde(
        rename = "stck_lwpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub low_price: Decimal,
    #[serde(rename = "lwpr_hour")]
    pub low_price_time: String,
    #[serde(rename = "acml_lwpr_date")]
    pub accumulated_low_price_date: String,
    #[serde(
        rename = "lwpr_vrss_prpr_rate",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub low_price_to_current_price_rate: Decimal,
    #[serde(
        rename = "dsgt_date_clpr_vrss_prpr_rate",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub target_date_close_to_current_price_rate: Decimal,
    #[serde(
        rename = "cnnt_ascn_dynu",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub continuous_rise_days: Decimal,
    #[serde(
        rename = "hgpr_vrss_prpr_rate",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub high_price_to_current_price_rate: Decimal,
    #[serde(
        rename = "cnnt_down_dynu",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub continuous_fall_days: Decimal,
    #[serde(rename = "oprc_vrss_prpr_sign")]
    pub open_price_difference_sign: String,
    #[serde(
        rename = "oprc_vrss_prpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub open_price_difference: Decimal,
    #[serde(
        rename = "oprc_vrss_prpr_rate",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub open_price_to_current_price_rate: Decimal,
    #[serde(rename = "prd_rsfl", deserialize_with = "deserialize_decimal_from_str")]
    pub period_fluctuation: Decimal,
    #[serde(
        rename = "prd_rsfl_rate",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub period_fluctuation_rate: Decimal,
}

fn deserialize_decimal_from_str<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireAskingPriceExpCcnOutput {
    pub asking_price: InquireAskingPriceExpCcnOutput1,
    pub expected_conclusion: InquireAskingPriceExpCcnOutput2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireAskingPriceExpCcnOutput1 {
    #[serde(rename = "askp1", deserialize_with = "deserialize_decimal_from_str")]
    pub ask_price1: Decimal,
    #[serde(rename = "bidp1", deserialize_with = "deserialize_decimal_from_str")]
    pub bid_price1: Decimal,
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireAskingPriceExpCcnOutput2 {
    #[serde(
        rename = "antc_cnpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub expected_conclusion_price: Decimal,
    #[serde(
        rename = "antc_cntg_vrss",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub expected_conclusion_difference: Decimal,
    #[serde(
        rename = "antc_cntg_vol",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub expected_conclusion_volume: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireCcnlItem {
    #[serde(rename = "stck_cntg_hour")]
    pub conclusion_time: String,
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
    #[serde(rename = "cntg_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub conclusion_volume: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireDailyPriceItem {
    #[serde(rename = "stck_bsop_date")]
    pub business_date: String,
    #[serde(
        rename = "stck_oprc",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub open_price: Decimal,
    #[serde(
        rename = "stck_hgpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub high_price: Decimal,
    #[serde(
        rename = "stck_lwpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub low_price: Decimal,
    #[serde(
        rename = "stck_clpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub close_price: Decimal,
    #[serde(rename = "acml_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub accumulated_volume: Decimal,
    #[serde(
        rename = "acml_tr_pbmn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub accumulated_trade_amount: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireDailyItemChartPriceOutput {
    pub summary: InquireDailyItemChartPriceSummary,
    pub prices: Vec<InquireDailyItemChartPriceItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireDailyItemChartPriceSummary {
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireDailyItemChartPriceItem {
    #[serde(rename = "stck_bsop_date")]
    pub business_date: String,
    #[serde(
        rename = "stck_oprc",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub open_price: Decimal,
    #[serde(
        rename = "stck_hgpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub high_price: Decimal,
    #[serde(
        rename = "stck_lwpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub low_price: Decimal,
    #[serde(
        rename = "stck_clpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub close_price: Decimal,
    #[serde(rename = "acml_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub accumulated_volume: Decimal,
    #[serde(
        rename = "acml_tr_pbmn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub accumulated_trade_amount: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquirePriceOutput {
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
        rename = "acml_tr_pbmn",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub accumulated_trade_amount: Decimal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireTimeItemChartPriceOutput {
    pub summary: InquireTimeItemChartPriceSummary,
    pub items: Vec<InquireTimeItemChartPriceItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireTimeItemChartPriceSummary {
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct InquireTimeItemChartPriceItem {
    #[serde(rename = "stck_cntg_hour")]
    pub conclusion_time: String,
    #[serde(
        rename = "stck_prpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub current_price: Decimal,
    #[serde(
        rename = "stck_oprc",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub open_price: Decimal,
    #[serde(
        rename = "stck_hgpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub high_price: Decimal,
    #[serde(
        rename = "stck_lwpr",
        deserialize_with = "deserialize_decimal_from_str"
    )]
    pub low_price: Decimal,
    #[serde(rename = "cntg_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub conclusion_volume: Decimal,
    #[serde(rename = "acml_vol", deserialize_with = "deserialize_decimal_from_str")]
    pub accumulated_volume: Decimal,
}

fn deserialize_decimal_from_str<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

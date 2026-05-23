use rust_decimal::Decimal;

use crate::error::{Error, Result};

const KIS_ORDER_NO_LEN: usize = 10;

pub(crate) struct CaretFields<'a> {
    fields: Vec<&'a str>,
}

impl<'a> CaretFields<'a> {
    pub(crate) fn new(payload: &'a str, field_count: usize, context: &'static str) -> Result<Self> {
        let fields = payload.split('^').collect::<Vec<_>>();

        if fields.len() != field_count {
            return Err(Error::parse(format!(
                "{context} field count mismatch: expected {field_count}, got {}",
                fields.len()
            )));
        }

        Ok(Self { fields })
    }

    pub(crate) fn new_range(
        payload: &'a str,
        min_field_count: usize,
        max_field_count: usize,
        context: &'static str,
    ) -> Result<Self> {
        let fields = payload.split('^').collect::<Vec<_>>();

        if fields.len() < min_field_count || fields.len() > max_field_count {
            return Err(Error::parse(format!(
                "{context} field count mismatch: expected {min_field_count}..={max_field_count}, got {}",
                fields.len()
            )));
        }

        Ok(Self { fields })
    }

    pub(crate) fn text(&self, index: usize) -> String {
        self.fields[index].to_string()
    }

    pub(crate) fn optional_decimal(
        &self,
        index: usize,
        context: &'static str,
    ) -> Result<Option<Decimal>> {
        parse_optional_decimal(self.fields[index], context)
    }

    pub(crate) fn get(&self, index: usize) -> Option<&'a str> {
        self.fields.get(index).copied()
    }
}

pub(crate) fn parse_optional_decimal(
    value: &str,
    context: &'static str,
) -> Result<Option<Decimal>> {
    if value.is_empty() {
        return Ok(None);
    }

    value
        .parse()
        .map(Some)
        .map_err(|error| Error::parse(format!("failed to parse {context}: {error}")))
}

pub(crate) fn normalize_kis_order_no(value: &str) -> String {
    if value.is_empty()
        || value.chars().all(|char| char == '0')
        || !value.chars().all(|char| char.is_ascii_digit())
        || value.len() >= KIS_ORDER_NO_LEN
    {
        return value.to_string();
    }

    format!("{value:0>KIS_ORDER_NO_LEN$}")
}

pub(crate) fn zero_pad_numeric_text(value: &str, len: usize) -> String {
    if value.is_empty() || !value.chars().all(|char| char.is_ascii_digit()) || value.len() >= len {
        return value.to_string();
    }

    format!("{value:0>len$}")
}

pub(crate) fn mask_tail(value: &str, visible_tail_len: usize) -> String {
    let char_count = value.chars().count();

    if char_count <= visible_tail_len {
        return "*".repeat(char_count);
    }

    let masked_len = char_count - visible_tail_len;
    let tail = value.chars().skip(masked_len).collect::<String>();

    format!("{}{}", "*".repeat(masked_len), tail)
}

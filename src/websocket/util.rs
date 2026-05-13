use rust_decimal::Decimal;

use crate::error::{Error, Result};

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

    pub(crate) fn optional_text(&self, index: usize) -> String {
        self.fields
            .get(index)
            .copied()
            .unwrap_or_default()
            .to_string()
    }

    pub(crate) fn optional_decimal(
        &self,
        index: usize,
        context: &'static str,
    ) -> Result<Option<Decimal>> {
        parse_optional_decimal(self.fields[index], context)
    }

    pub(crate) fn optional_decimal_at(
        &self,
        index: usize,
        context: &'static str,
    ) -> Result<Option<Decimal>> {
        let Some(value) = self.fields.get(index) else {
            return Ok(None);
        };

        parse_optional_decimal(value, context)
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

pub(crate) fn mask_tail(value: &str, visible_tail_len: usize) -> String {
    let char_count = value.chars().count();

    if char_count <= visible_tail_len {
        return "*".repeat(char_count);
    }

    let masked_len = char_count - visible_tail_len;
    let tail = value.chars().skip(masked_len).collect::<String>();

    format!("{}{}", "*".repeat(masked_len), tail)
}

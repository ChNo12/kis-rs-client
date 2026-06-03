use crate::auth::AccessToken;
use crate::config::{Account, Environment};
use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::rest::domestic_stock::common::{
    Endpoint, get, post, require_output, require_output_pair,
};
use crate::rest::domestic_stock::trading::{DoubleOutputResponse, Service, SingleOutputResponse};

pub(crate) const CANO: &str = "CANO";
pub(crate) const ACNT_PRDT_CD: &str = "ACNT_PRDT_CD";
pub(crate) const PDNO: &str = "PDNO";
pub(crate) const ORD_DVSN: &str = "ORD_DVSN";
pub(crate) const ORD_QTY: &str = "ORD_QTY";
pub(crate) const ORD_UNPR: &str = "ORD_UNPR";
pub(crate) const EXCG_ID_DVSN_CD: &str = "EXCG_ID_DVSN_CD";
pub(crate) const SLL_TYPE: &str = "SLL_TYPE";
pub(crate) const CNDT_PRIC: &str = "CNDT_PRIC";
pub(crate) const KRX_FWDG_ORD_ORGNO: &str = "KRX_FWDG_ORD_ORGNO";
pub(crate) const ORGN_ODNO: &str = "ORGN_ODNO";
pub(crate) const RVSE_CNCL_DVSN_CD: &str = "RVSE_CNCL_DVSN_CD";
pub(crate) const QTY_ALL_ORD_YN: &str = "QTY_ALL_ORD_YN";
pub(crate) const INQR_DVSN_1: &str = "INQR_DVSN_1";
pub(crate) const INQR_DVSN_2: &str = "INQR_DVSN_2";
pub(crate) const CTX_AREA_FK100: &str = "CTX_AREA_FK100";
pub(crate) const CTX_AREA_NK100: &str = "CTX_AREA_NK100";
pub(crate) const INQR_STRT_DT: &str = "INQR_STRT_DT";
pub(crate) const INQR_END_DT: &str = "INQR_END_DT";
pub(crate) const SLL_BUY_DVSN_CD: &str = "SLL_BUY_DVSN_CD";
pub(crate) const CCLD_DVSN: &str = "CCLD_DVSN";
pub(crate) const INQR_DVSN: &str = "INQR_DVSN";
pub(crate) const INQR_DVSN_3: &str = "INQR_DVSN_3";
pub(crate) const ORD_GNO_BRNO: &str = "ORD_GNO_BRNO";
pub(crate) const ODNO: &str = "ODNO";
pub(crate) const AFHR_FLPR_YN: &str = "AFHR_FLPR_YN";
pub(crate) const OFL_YN: &str = "OFL_YN";
pub(crate) const UNPR_DVSN: &str = "UNPR_DVSN";
pub(crate) const FUND_STTL_ICLD_YN: &str = "FUND_STTL_ICLD_YN";
pub(crate) const FNCG_AMT_AUTO_RDPT_YN: &str = "FNCG_AMT_AUTO_RDPT_YN";
pub(crate) const PRCS_DVSN: &str = "PRCS_DVSN";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviseCancel {
    Revise,
    Cancel,
}

impl ReviseCancel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Revise => "01",
            Self::Cancel => "02",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllQuantityOrder {
    All,
    Partial,
}

impl AllQuantityOrder {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "Y",
            Self::Partial => "N",
        }
    }
}

pub(crate) async fn get_output<T: HttpClient>(
    service: &Service<'_, T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    params: Vec<(&'static str, String)>,
    continuation: Option<&crate::rest::domestic_stock::Continuation>,
    parse_context: &'static str,
) -> Result<SingleOutputResponse> {
    let response = get(
        service.client,
        access_token,
        endpoint,
        params,
        continuation,
        parse_context,
    )
    .await?;
    let (output, continuation) = require_output(response, parse_context)?;

    Ok(SingleOutputResponse {
        output,
        continuation,
    })
}

pub(crate) async fn get_output_pair<T: HttpClient>(
    service: &Service<'_, T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    params: Vec<(&'static str, String)>,
    continuation: Option<&crate::rest::domestic_stock::Continuation>,
    parse_context: &'static str,
) -> Result<DoubleOutputResponse> {
    let response = get(
        service.client,
        access_token,
        endpoint,
        params,
        continuation,
        parse_context,
    )
    .await?;
    let (output1, output2, continuation) = require_output_pair(response, parse_context)?;

    Ok(DoubleOutputResponse {
        output1,
        output2,
        continuation,
    })
}

pub(crate) async fn post_output<T: HttpClient>(
    service: &Service<'_, T>,
    access_token: &AccessToken,
    endpoint: Endpoint,
    body_params: Vec<(&'static str, String)>,
    parse_context: &'static str,
) -> Result<SingleOutputResponse> {
    service.client.config().require_ordering_allowed()?;

    let response = post(
        service.client,
        access_token,
        endpoint,
        body_params,
        parse_context,
    )
    .await?;
    let (output, continuation) = require_output(response, parse_context)?;

    Ok(SingleOutputResponse {
        output,
        continuation,
    })
}

pub(crate) fn account_params(account: &Account) -> Vec<(&'static str, String)> {
    vec![
        (CANO, account.number.as_str().to_string()),
        (ACNT_PRDT_CD, account.product_code.as_str().to_string()),
    ]
}

pub(crate) fn require_account_params(
    service: &Service<'_, impl HttpClient>,
) -> Result<Vec<(&'static str, String)>> {
    Ok(account_params(service.client.config().require_account()?))
}

pub(crate) fn env_tr_id(
    environment: Environment,
    real_tr_id: &'static str,
    virtual_tr_id: &'static str,
) -> &'static str {
    match environment {
        Environment::Real => real_tr_id,
        Environment::Virtual => virtual_tr_id,
    }
}

pub(crate) fn require_non_empty(
    value: impl Into<String>,
    field_name: &'static str,
) -> Result<String> {
    let value = value.into();

    if value.is_empty() {
        return Err(Error::config(format!("{field_name} is empty")));
    }

    Ok(value)
}
